use std::{
    net::SocketAddr,
    num::NonZero,
    sync::{Arc, Weak},
};

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
    net::{
        EncryptionError, GameProfile, PacketHandlerResult, PacketRateLimiter, PlayerConfig,
        can_not_join,
    },
    server::Server,
};

use super::JavaClient;
use crate::net::user::{Edition, OutgoingPacket, User};
use crate::plugin::server::packet::{
    PacketReceivedEvent, PacketSentEvent, decode_packet, encode_packet,
};

const BRAND_CHANNEL_PREFIX: &str = "minecraft:brand";

/// How long a connection may stay silent before login finishes.
///
/// Once a player is in game, [`JavaClient::progress_player_packets`] keeps the
/// connection honest with keep-alives. Nothing plays that role beforehand, and
/// accepted sockets have no TCP keep-alive either, so a peer that stops talking
/// without closing would otherwise hold its descriptor for the lifetime of the
/// server. The timer covers silence rather than the whole handshake: it is reset
/// on every packet, so a slow but progressing login is never cut off.
const HANDSHAKE_IDLE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

pub struct PendingConnection {
    pub user: Arc<User>,
    pub(crate) outgoing: Option<tokio::sync::mpsc::UnboundedReceiver<OutgoingPacket>>,
    pub id: u64,
    pub address: SocketAddr,
    pub server_address: String,
    pub version: Arc<AtomicCell<JavaMinecraftVersion>>,
    pub connection_state: Arc<AtomicCell<ConnectionState>>,
    pub close_token: CancellationToken,
    pub network_writer: TCPNetworkEncoder<BufWriter<OwnedWriteHalf>>,
    pub network_reader: TCPNetworkDecoder<BufReader<OwnedReadHalf>>,
    pub gameprofile: Option<GameProfile>,
    pub config: Option<PlayerConfig>,
    pub brand: Option<String>,
    pub packet_limiter: PacketRateLimiter,
    pub verify_token: Option<[u8; 4]>,
}

impl PendingConnection {
    #[must_use]
    pub fn new(
        tcp_stream: TcpStream,
        address: SocketAddr,
        id: u64,
        packet_limiter: PacketRateLimiter,
        server: Weak<Server>,
    ) -> Self {
        let (read, write) = tcp_stream.into_split();
        let user = User::new(address, Edition::Java, server);
        user.java_version.store(CURRENT_MC_VERSION);
        Self {
            outgoing: user.take_outgoing(),
            version: user.java_version.clone(),
            connection_state: user.decoder_state.clone(),
            close_token: user.close_token.clone(),
            user,
            id,
            address,
            server_address: String::new(),
            network_writer: TCPNetworkEncoder::new(BufWriter::new(write)),
            network_reader: TCPNetworkDecoder::new(BufReader::new(read)),
            gameprofile: None,
            config: None,
            brand: None,
            packet_limiter,
            verify_token: None,
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
        let deadline = tokio::time::Instant::now() + HANDSHAKE_IDLE_TIMEOUT;
        self.sync_user_info();
        let user = self.user.clone();
        let compression = self.network_reader.compression_state();
        let read_packet = self.network_reader.get_raw_packet();
        tokio::pin!(read_packet);
        loop {
            let result = tokio::select! {
                () = close_token.cancelled() => return None,
                () = tokio::time::sleep_until(deadline) => return None,
                outgoing = async {
                    match self.outgoing.as_mut() {
                        Some(receiver) => receiver.recv().await,
                        None => std::future::pending().await,
                    }
                } => {
                    if let Some(packet) = outgoing {
                        let len = packet.data.len();
                        let _ = Self::write_raw_packet(&user, &mut self.network_writer, &compression, packet.data, packet.silent).await;
                        crate::net::decrement_pending_bytes(&self.user.pending_bytes, len);
                        if let Some(completion) = packet.completion { let _ = completion.send(()); }
                    }
                    continue;
                },
                result = &mut read_packet => result,
            };
            return match result {
                Ok(packet) => Some(packet),
                Err(error) => {
                    if !matches!(error, PacketDecodeError::ConnectionClosed) {
                        debug!("Failed to decode packet from client {}: {error}", self.id);
                    }
                    user.close();
                    None
                }
            };
        }
    }

    pub async fn send_packet_now<P: ClientPacket>(&mut self, packet: &P) {
        let _ = self.send_packet_checked(packet).await;
    }

    pub(super) async fn send_packet_checked<P: ClientPacket>(
        &mut self,
        packet: &P,
    ) -> Option<RawPacket> {
        while let Some(queued) = self
            .outgoing
            .as_mut()
            .and_then(|queue| queue.try_recv().ok())
        {
            let len = queued.data.len();
            let _ = self.send_raw_packet_now(queued.data, queued.silent).await;
            crate::net::decrement_pending_bytes(&self.user.pending_bytes, len);
            if let Some(completion) = queued.completion {
                let _ = completion.send(());
            }
        }
        let mut packet_buf = Vec::new();
        let state = self.user.encoder_state.load();
        let expected = P::state();
        if self.is_closed()
            || (state != expected
                && !matches!(
                    (state, expected),
                    (
                        ConnectionState::Login | ConnectionState::Transfer,
                        ConnectionState::Login | ConnectionState::Transfer
                    )
                ))
        {
            return None;
        }
        if let Err(err) =
            JavaClient::write_packet_for_version(packet, self.version.load(), &mut packet_buf)
        {
            error!("Failed to write packet: {err:?}");
            return None;
        }
        self.send_raw_packet_now(packet_buf.into(), false).await
    }

    fn sync_user_info(&self) {
        self.user.update_info(|info| {
            info.address = self.address;
            info.server_address.clone_from(&self.server_address);
            info.profile.clone_from(&self.gameprofile);
            info.config.clone_from(&self.config);
            info.brand.clone_from(&self.brand);
        });
    }

    async fn send_raw_packet_now(&mut self, data: Bytes, silent: bool) -> Option<RawPacket> {
        self.sync_user_info();
        Self::write_raw_packet(
            &self.user,
            &mut self.network_writer,
            &self.network_reader.compression_state(),
            data,
            silent,
        )
        .await
    }

    async fn write_raw_packet(
        user: &Arc<User>,
        writer: &mut TCPNetworkEncoder<BufWriter<OwnedWriteHalf>>,
        compression: &pumpkin_protocol::java::packet_decoder::CompressionState,
        data: Bytes,
        silent: bool,
    ) -> Option<RawPacket> {
        let packet = decode_packet(data).ok()?;
        let packet = PacketSentEvent::filter(user, packet, silent).await?;
        let compression_threshold = if matches!(
            user.encoder_state.load(),
            ConnectionState::Login | ConnectionState::Transfer
        ) && packet.id
            == pumpkin_protocol::java::client::login::CSetCompression::to_id(
                user.java_version.load(),
            ) {
            let mut body = packet.payload.as_ref();
            match pumpkin_protocol::codec::var_int::VarInt::decode(&mut body) {
                Ok(threshold) if body.is_empty() => Some(threshold.0),
                _ => {
                    user.close();
                    return None;
                }
            }
        } else {
            None
        };
        let bytes = encode_packet(packet.id, &packet.payload).ok()?;
        if let Err(err) = writer.write_packet(bytes).await {
            warn!("Failed to send packet to client {}: {err}", user.id);
            user.close();
            return None;
        }
        if writer.flush().await.is_err() {
            user.close();
            return None;
        }
        if let Some(threshold) = compression_threshold {
            if threshold < 0 {
                compression.set(None);
                writer.disable_compression();
            } else {
                let level = user.server.upgrade().map_or(4, |server| {
                    server
                        .advanced_config
                        .networking
                        .java
                        .compression
                        .info
                        .level
                });
                compression.set(Some(threshold as usize));
                writer.set_compression((threshold as usize, level));
            }
        }
        super::advance_encoder_state(user, packet.id);
        Some(packet)
    }

    pub async fn kick(&mut self, reason: TextComponent) {
        match self.user.encoder_state.load() {
            ConnectionState::Login | ConnectionState::Transfer => {
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
            if !self.packet_limiter.check_packet() {
                warn!(
                    "Pending client {} exceeded packet rate limit (rate: {}/s)",
                    self.id,
                    self.packet_limiter.max_rate()
                );
                self.kick(TextComponent::text(
                    server
                        .advanced_config
                        .networking
                        .java
                        .packet_limiter
                        .kick_message
                        .clone(),
                ))
                .await;
                return PacketHandlerResult::Stop;
            }

            self.sync_user_info();
            let encrypted_response = matches!(
                self.connection_state.load(),
                ConnectionState::Login | ConnectionState::Transfer
            ) && packet.id
                == pumpkin_protocol::java::server::login::SEncryptionResponse::to_id(
                    self.version.load(),
                );
            let Some(packet) = PacketReceivedEvent::filter(&self.user, packet).await else {
                if encrypted_response {
                    self.close();
                    return PacketHandlerResult::Stop;
                }
                continue;
            };
            if self.is_closed() {
                return PacketHandlerResult::Stop;
            }
            if encrypted_response
                && packet.id
                    != pumpkin_protocol::java::server::login::SEncryptionResponse::to_id(
                        self.version.load(),
                    )
            {
                self.close();
                return PacketHandlerResult::Stop;
            }
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
        server: &Arc<Server>,
        packet: &RawPacket,
    ) -> Result<Option<PacketHandlerResult>, ReadingError> {
        debug!("Handling login group");
        let mut payload = &packet.payload[..];
        let version = self.version.load();

        match packet.id {
            id if id == pumpkin_protocol::java::server::login::SLoginStart::to_id(version) => {
                Ok(self
                    .handle_login_start(
                        server,
                        pumpkin_protocol::java::server::login::SLoginStart::read(
                            &mut payload,
                            &version,
                        )?,
                    )
                    .await)
            }
            id if id
                == pumpkin_protocol::java::server::login::SEncryptionResponse::to_id(version) =>
            {
                Ok(self
                    .handle_encryption_response(
                        server,
                        pumpkin_protocol::java::server::login::SEncryptionResponse::read(
                            &mut payload,
                            &version,
                        )?,
                    )
                    .await)
            }
            id if id
                == pumpkin_protocol::java::server::login::SLoginPluginResponse::to_id(version) =>
            {
                Ok(self
                    .handle_plugin_response(
                        server,
                        pumpkin_protocol::java::server::login::SLoginPluginResponse::read(
                            &mut payload,
                            &version,
                        )?,
                    )
                    .await)
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
                if !self.user.acknowledge_state(ConnectionState::Config) {
                    return Err(ReadingError::Message(
                        "Unexpected login acknowledgement".into(),
                    ));
                }
                Ok(self.handle_login_acknowledged(server).await)
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
                let _ = SAcknowledgeFinishConfig::read(&mut payload, &version)?;
                if !self.user.acknowledge_state(ConnectionState::Play) {
                    return Err(ReadingError::Message(
                        "Unexpected configuration acknowledgement".into(),
                    ));
                }
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
                self.handle_known_packs().await;
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
