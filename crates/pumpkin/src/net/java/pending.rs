use std::{net::SocketAddr, num::NonZero, sync::Arc};

use bytes::Bytes;
use crossbeam::atomic::AtomicCell;
use pumpkin_config::networking::compression::CompressionInfo;
use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_protocol::{
    ClientPacket, ConnectionState, PacketDecodeError, RawPacket, ServerPacket,
    java::{
        client::config::CConfigDisconnect,
        client::login::CLoginDisconnect,
        client::play::CPlayDisconnect,
        packet_decoder::TCPNetworkDecoder,
        packet_encoder::TCPNetworkEncoder,
        server::config::{
            SAcceptCodeOfConduct, SAcknowledgeFinishConfig, SClientInformationConfig,
            SConfigCookieResponse, SConfigPong, SConfigResourcePack, SKnownPacks, SPluginMessage,
        },
    },
    packet::MultiVersionJavaPacket,
    ser::ReadingError,
};
use pumpkin_util::{Hand, text::TextComponent, version::JavaMinecraftVersion};
use tokio::{
    io::{BufReader, BufWriter},
    net::{
        TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, warn};

use crate::{
    entity::player::ChatMode,
    net::{EncryptionError, GameProfile, PacketHandlerResult, PlayerConfig, can_not_join},
    server::Server,
};

use super::JavaClient;

const BRAND_CHANNEL_PREFIX: &str = "minecraft:brand";

pub struct PendingConnection {
    pub id: u64,
    pub address: SocketAddr,
    pub server_address: String,
    pub version: AtomicCell<JavaMinecraftVersion>,
    pub connection_state: AtomicCell<ConnectionState>,
    pub close_token: CancellationToken,
    pub network_writer: TCPNetworkEncoder<BufWriter<OwnedWriteHalf>>,
    pub network_reader: TCPNetworkDecoder<BufReader<OwnedReadHalf>>,
    pub gameprofile: Option<GameProfile>,
    pub config: Option<PlayerConfig>,
    pub brand: Option<String>,
}

impl PendingConnection {
    #[must_use]
    pub fn new(tcp_stream: TcpStream, address: SocketAddr, id: u64) -> Self {
        let (read, write) = tcp_stream.into_split();
        Self {
            id,
            address,
            server_address: String::new(),
            version: AtomicCell::new(CURRENT_MC_VERSION),
            connection_state: AtomicCell::new(ConnectionState::HandShake),
            close_token: CancellationToken::new(),
            network_writer: TCPNetworkEncoder::new(BufWriter::new(write)),
            network_reader: TCPNetworkDecoder::new(BufReader::new(read)),
            gameprofile: None,
            config: None,
            brand: None,
        }
    }

    pub fn close(&self) {
        self.close_token.cancel();
    }

    pub fn is_closed(&self) -> bool {
        self.close_token.is_cancelled()
    }

    pub async fn await_close_interrupt(&self) {
        self.close_token.cancelled().await;
    }

    pub fn set_encryption(&mut self, shared_secret: &[u8]) -> Result<(), EncryptionError> {
        let crypt_key: [u8; 16] = shared_secret
            .try_into()
            .map_err(|_| EncryptionError::SharedWrongLength)?;
        self.network_reader
            .set_encryption(&crypt_key)
            .map_err(|_| EncryptionError::AlreadyEncrypted)?;
        self.network_writer
            .set_encryption(&crypt_key)
            .map_err(|_| EncryptionError::AlreadyEncrypted)?;
        Ok(())
    }

    pub fn set_compression(&mut self, compression: &CompressionInfo) {
        if compression.level > 9 {
            error!("Invalid compression level! Clients will not be able to read this!");
        }

        self.network_reader
            .set_compression(compression.threshold as usize);

        self.network_writer
            .set_compression((compression.threshold as usize, compression.level));
    }

    pub async fn get_packet(&mut self) -> Option<RawPacket> {
        let close_token = self.close_token.clone();
        let packet_result = tokio::select! {
            () = close_token.cancelled() => {
                debug!("Canceling pending connection packet processing");
                return None;
            },
            res = self.network_reader.get_raw_packet() => res,
        };

        match packet_result {
            Ok(packet) => Some(packet),
            Err(err) => {
                if !matches!(err, PacketDecodeError::ConnectionClosed) {
                    debug!("Failed to decode packet from client {}: {}", self.id, err);
                    let text = format!("Error while reading incoming packet {err}");
                    self.kick(TextComponent::text(text)).await;
                }
                None
            }
        }
    }

    pub async fn send_packet_now<P: ClientPacket>(&mut self, packet: &P) {
        let mut packet_buf = Vec::new();
        if let Err(err) =
            JavaClient::write_packet_for_version(packet, self.version.load(), &mut packet_buf)
        {
            error!("Failed to write packet: {err:?}");
            return;
        }
        let payload = Bytes::from(packet_buf);
        if let Err(err) = self.network_writer.write_packet(payload).await {
            warn!("Failed to send packet to client {}: {}", self.id, err);
        }
        let _ = self.network_writer.flush().await;
    }

    pub async fn kick(&mut self, reason: TextComponent) {
        match self.connection_state.load() {
            ConnectionState::Login => {
                self.send_packet_now(&CLoginDisconnect::new(
                    serde_json::to_string(&reason.0).unwrap_or_else(|_| String::new()),
                ))
                .await;
            }
            ConnectionState::Config => {
                self.send_packet_now(&CConfigDisconnect::new(&reason.get_text()))
                    .await;
            }
            ConnectionState::Play => {
                self.send_packet_now(&CPlayDisconnect::new(&reason)).await;
            }
            _ => {}
        }
        debug!("Closing connection for {}", self.id);
        self.close();
    }

    pub async fn handle_login_sequence(&mut self, server: &Arc<Server>) -> PacketHandlerResult {
        while let Some(packet) = self.get_packet().await {
            match self.handle_packet(server, &packet).await {
                Ok(result) => {
                    if let Some(result) = result {
                        return result;
                    }
                }
                Err(error) => {
                    let text = format!("Error while reading incoming packet {error}");
                    debug!(
                        "Failed to read incoming packet with id {}: {}",
                        packet.id, error
                    );
                    self.kick(TextComponent::text(text)).await;
                }
            }
        }
        PacketHandlerResult::Stop
    }

    pub async fn handle_packet(
        &mut self,
        server: &Arc<Server>,
        packet: &RawPacket,
    ) -> Result<Option<PacketHandlerResult>, ReadingError> {
        match self.connection_state.load() {
            ConnectionState::HandShake => self.handle_handshake_packet(packet).await,
            ConnectionState::Status => self.handle_status_packet(server, packet).await,
            ConnectionState::Login | ConnectionState::Transfer => {
                self.handle_login_packet(server, packet).await
            }
            ConnectionState::Config => self.handle_config_packet(server, packet).await,
            ConnectionState::Play => Ok(None),
        }
    }

    async fn handle_handshake_packet(
        &mut self,
        packet: &RawPacket,
    ) -> Result<Option<PacketHandlerResult>, ReadingError> {
        debug!("Handling handshake group");
        let mut payload = &packet.payload[..];
        match packet.id {
            0 => {
                self.handle_handshake(pumpkin_protocol::java::server::handshake::SHandShake::read(
                    &mut payload,
                    &self.version.load(),
                )?)
                .await;
                Ok(None)
            }
            _ => Err(ReadingError::Message(format!(
                "Failed to handle packet id {} in Handshake State",
                packet.id
            ))),
        }
    }

    async fn handle_status_packet(
        &mut self,
        server: &Arc<Server>,
        packet: &RawPacket,
    ) -> Result<Option<PacketHandlerResult>, ReadingError> {
        debug!("Handling status group");
        let mut payload = &packet.payload[..];
        let version = self.version.load();

        match packet.id {
            id if id == pumpkin_protocol::java::server::status::SStatusRequest::to_id(version) => {
                self.handle_status_request(server).await;
                Ok(None)
            }
            id if id
                == pumpkin_protocol::java::server::status::SStatusPingRequest::to_id(version) =>
            {
                self.handle_ping_request(
                    pumpkin_protocol::java::server::status::SStatusPingRequest::read(
                        &mut payload,
                        &version,
                    )?,
                )
                .await;
                Ok(None)
            }
            _ => Err(ReadingError::Message(format!(
                "Failed to handle java client packet id {} in Status State",
                packet.id
            ))),
        }
    }

    async fn handle_login_packet(
        &mut self,
        server: &Server,
        packet: &RawPacket,
    ) -> Result<Option<PacketHandlerResult>, ReadingError> {
        debug!("Handling login group");
        let mut payload = &packet.payload[..];
        let version = self.version.load();

        match packet.id {
            id if id == pumpkin_protocol::java::server::login::SLoginStart::to_id(version) => {
                self.handle_login_start(
                    server,
                    pumpkin_protocol::java::server::login::SLoginStart::read(
                        &mut payload,
                        &version,
                    )?,
                )
                .await;
                Ok(None)
            }
            id if id
                == pumpkin_protocol::java::server::login::SEncryptionResponse::to_id(version) =>
            {
                self.handle_encryption_response(
                    server,
                    pumpkin_protocol::java::server::login::SEncryptionResponse::read(
                        &mut payload,
                        &version,
                    )?,
                )
                .await;
                Ok(None)
            }
            id if id
                == pumpkin_protocol::java::server::login::SLoginPluginResponse::to_id(version) =>
            {
                self.handle_plugin_response(
                    server,
                    pumpkin_protocol::java::server::login::SLoginPluginResponse::read(
                        &mut payload,
                        &version,
                    )?,
                )
                .await;
                Ok(None)
            }
            id if id
                == pumpkin_protocol::java::server::login::SLoginCookieResponse::to_id(version) =>
            {
                self.handle_login_cookie_response(
                    &pumpkin_protocol::java::server::login::SLoginCookieResponse::read(
                        &mut payload,
                        &version,
                    )?,
                );
                Ok(None)
            }
            id if id
                == pumpkin_protocol::java::server::login::SLoginAcknowledged::to_id(version) =>
            {
                self.handle_login_acknowledged(server).await;
                Ok(None)
            }
            _ => Err(ReadingError::Message(format!(
                "Failed to handle packet id {} in Login State",
                packet.id
            ))),
        }
    }

    async fn handle_config_packet(
        &mut self,
        server: &Arc<Server>,
        packet: &RawPacket,
    ) -> Result<Option<PacketHandlerResult>, ReadingError> {
        debug!("Handling config group");
        let mut payload = &packet.payload[..];
        let version = self.version.load();

        match packet.id {
            id if id == SClientInformationConfig::to_id(version) => {
                self.handle_client_information_config(SClientInformationConfig::read(
                    &mut payload,
                    &version,
                )?)
                .await;
                Ok(None)
            }
            id if id == SPluginMessage::to_id(version) => {
                self.handle_plugin_message(SPluginMessage::read(&mut payload, &version)?)
                    .await;
                Ok(None)
            }
            id if id == SAcknowledgeFinishConfig::to_id(version) => {
                let Some(profile) = self.gameprofile.clone() else {
                    return Ok(Some(PacketHandlerResult::Stop));
                };
                let config = self.config.clone().unwrap_or_default();
                self.connection_state.store(ConnectionState::Play);
                if let Some(reason) = can_not_join(&profile, &self.address, server).await {
                    self.kick(reason).await;
                    Ok(Some(PacketHandlerResult::Stop))
                } else {
                    Ok(Some(PacketHandlerResult::ReadyToPlay(profile, config)))
                }
            }
            id if id == SKnownPacks::to_id(version) => {
                self.handle_known_packs(SKnownPacks::read(&mut payload, &version)?, server)
                    .await;
                Ok(None)
            }
            id if id == SConfigResourcePack::to_id(version) => {
                self.handle_resource_pack_response(
                    server,
                    SConfigResourcePack::read(&mut payload, &version)?,
                )
                .await;
                Ok(None)
            }
            id if id == SConfigCookieResponse::to_id(version) => {
                self.handle_config_cookie_response(&SConfigCookieResponse::read(
                    &mut payload,
                    &version,
                )?);
                Ok(None)
            }
            id if id == SConfigPong::to_id(version) => {
                let _pong = SConfigPong::read(&mut payload, &version)?;
                Ok(None)
            }
            id if id == SAcceptCodeOfConduct::to_id(version) => {
                let _accept = SAcceptCodeOfConduct::read(&mut payload, &version)?;
                Ok(None)
            }
            _ => Err(ReadingError::Message(format!(
                "Failed to handle packet id {} in Config State",
                packet.id
            ))),
        }
    }

    pub async fn handle_client_information_config(
        &mut self,
        client_information: SClientInformationConfig<'_>,
    ) {
        debug!("Handling client settings");
        if client_information.view_distance <= 0 {
            self.kick(TextComponent::text(
                "Cannot have zero or negative view distance!",
            ))
            .await;
            return;
        }

        if let (Ok(main_hand), Ok(chat_mode)) = (
            Hand::try_from(client_information.main_hand.0),
            ChatMode::try_from(client_information.chat_mode.0),
        ) {
            self.config = Some(PlayerConfig {
                locale: client_information.locale.to_string(),
                view_distance: NonZero::new(client_information.view_distance as u8)
                    .unwrap_or(NonZero::<u8>::MIN),
                chat_mode,
                chat_colors: client_information.chat_colors,
                skin_parts: client_information.skin_parts,
                main_hand,
                text_filtering: client_information.text_filtering,
                server_listing: client_information.server_listing,
            });
        } else {
            self.kick(TextComponent::text("Invalid hand or chat type"))
                .await;
        }
    }

    pub async fn handle_plugin_message(&mut self, plugin_message: SPluginMessage<'_>) {
        debug!("Handling plugin message");
        if plugin_message.channel.starts_with(BRAND_CHANNEL_PREFIX) {
            debug!("Got a client brand");
            match core::str::from_utf8(plugin_message.data) {
                Ok(brand) => self.brand = Some(brand.to_string()),
                Err(e) => self.kick(TextComponent::text(e.to_string())).await,
            }
        }
    }

    pub async fn handle_resource_pack_response(
        &mut self,
        server: &Server,
        packet: SConfigResourcePack,
    ) {
        let resource_config = &server.advanced_config.resource_pack.java;
        if resource_config.enabled {
            use pumpkin_protocol::java::server::config::ResourcePackResponseResult;
            match packet.response_result() {
                ResourcePackResponseResult::Downloaded
                | ResourcePackResponseResult::DownloadSuccess
                | ResourcePackResponseResult::Accepted
                | ResourcePackResponseResult::Discarded
                | ResourcePackResponseResult::Unknown(_) => {}
                ResourcePackResponseResult::Declined => {
                    if resource_config.force {
                        self.kick(TextComponent::text("Required resource pack was declined"))
                            .await;
                    }
                }
                ResourcePackResponseResult::DownloadFail => {
                    if resource_config.force {
                        self.kick(TextComponent::text("Failed to download resource pack"))
                            .await;
                    }
                }
                ResourcePackResponseResult::InvalidUrl => {
                    self.kick(TextComponent::text("Invalid resource pack URL"))
                        .await;
                }
                ResourcePackResponseResult::ReloadFailed => {
                    self.kick(TextComponent::text("Failed to reload resource pack"))
                        .await;
                }
            }
        }
    }

    pub fn handle_config_cookie_response(&self, packet: &SConfigCookieResponse<'_>) {
        debug!(
            "Received cookie_response[config]: key: \"{}\", payload_length: \"{:?}\"",
            packet.key,
            packet.payload.as_ref().map(|p| p.len())
        );
    }
}
