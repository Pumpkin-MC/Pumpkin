use std::{io::Write, sync::Arc};

use bytes::Bytes;
use pumpkin_config::networking::compression::CompressionInfo;
use pumpkin_protocol::{
    ClientPacket, ConnectionState, PacketDecodeError, PacketEncodeError, RawPacket, ServerPacket,
    codec::var_int::VarInt,
    java::{
        client::{config::CConfigDisconnect, login::CLoginDisconnect, play::CPlayDisconnect},
        packet_decoder::TCPNetworkDecoder,
        packet_encoder::TCPNetworkEncoder,
        server::{
            config::{
                SAcknowledgeFinishConfig, SClientInformationConfig, SConfigCookieResponse,
                SConfigResourcePack, SKnownPacks, SPluginMessage,
            },
            handshake::SHandShake,
            login::{
                SEncryptionResponse, SLoginAcknowledged, SLoginCookieResponse,
                SLoginPluginResponse, SLoginStart,
            },
            status::{SStatusPingRequest, SStatusRequest},
        },
    },
    packet::Packet,
    ser::{NetworkWriteExt, ReadingError, WritingError},
};
use pumpkin_util::text::TextComponent;
use tokio::{
    io::{BufReader, BufWriter},
    net::{
        TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
    sync::Mutex,
};

pub mod config;
pub mod handshake;
pub mod login;
pub mod play;
pub mod status;

use crate::{
    net::{Client, EncryptionError},
    server::Server,
};

pub struct JavaClientPlatform {
    /// The packet encoder for outgoing packets.
    network_writer: Arc<Mutex<TCPNetworkEncoder<BufWriter<OwnedWriteHalf>>>>,
    /// The packet decoder for incoming packets.
    network_reader: Mutex<TCPNetworkDecoder<BufReader<OwnedReadHalf>>>,
}

impl JavaClientPlatform {
    #[must_use]
    pub fn new(tcp_stream: TcpStream) -> Self {
        let (read, write) = tcp_stream.into_split();
        Self {
            network_writer: Arc::new(Mutex::new(TCPNetworkEncoder::new(BufWriter::new(write)))),
            network_reader: Mutex::new(TCPNetworkDecoder::new(BufReader::new(read))),
        }
    }
    pub async fn set_encryption(
        &self,
        shared_secret: &[u8], // decrypted
    ) -> Result<(), EncryptionError> {
        let crypt_key: [u8; 16] = shared_secret
            .try_into()
            .map_err(|_| EncryptionError::SharedWrongLength)?;
        self.network_reader.lock().await.set_encryption(&crypt_key);
        self.network_writer.lock().await.set_encryption(&crypt_key);
        Ok(())
    }

    pub async fn set_compression(&self, compression: CompressionInfo) {
        if compression.level > 9 {
            log::error!("Invalid compression level! Clients will not be able to read this!");
        }

        self.network_reader
            .lock()
            .await
            .set_compression(compression.threshold as usize);

        self.network_writer
            .lock()
            .await
            .set_compression((compression.threshold as usize, compression.level));
    }

    /// Processes all packets received from the connected client in a loop.
    ///
    /// This function continuously dequeues packets from the client's packet queue and processes them.
    /// Processing involves calling the `handle_packet` function with the server instance and the packet itself.
    ///
    /// The loop exits when:
    ///
    /// - The connection is closed (checked before processing each packet).
    /// - An error occurs while processing a packet (client is kicked with an error message).
    ///
    /// # Arguments
    ///
    /// * `server`: A reference to the `Server` instance.
    pub async fn process_packets(&self, client: &Client, server: &Server) {
        while !client
            .make_player
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            let packet = self.get_packet(client).await;
            let Some(packet) = packet else { break };

            if let Err(error) = Self::handle_packet(client, server, &packet).await {
                let text = format!("Error while reading incoming packet {error}");
                log::error!(
                    "Failed to read incoming packet with id {}: {}",
                    packet.id,
                    error
                );
                self.kick(client, TextComponent::text(text)).await;
            }
        }
    }

    pub async fn get_packet(&self, client: &Client) -> Option<RawPacket> {
        let mut network_reader = self.network_reader.lock().await;
        tokio::select! {
            () = client.await_close_interrupt() => {
                log::debug!("Canceling player packet processing");
                None
            },
            packet_result = network_reader.get_raw_packet() => {
                match packet_result {
                    Ok(packet) => Some(packet),
                    Err(err) => {
                        if !matches!(err, PacketDecodeError::ConnectionClosed) {
                            log::warn!("Failed to decode packet from client {}: {}", client.id, err);
                            let text = format!("Error while reading incoming packet {err}");
                            self.kick(client, TextComponent::text(text)).await;
                        }
                        None
                    }
                }
            }
        }
    }

    pub async fn kick(&self, client: &Client, reason: TextComponent) {
        match client.connection_state.load() {
            ConnectionState::Login => {
                // TextComponent implements Serialize and writes in bytes instead of String, that's the reasib we only use content
                client
                    .send_packet_now(&CLoginDisconnect::new(
                        &serde_json::to_string(&reason.0).unwrap_or_else(|_| String::new()),
                    ))
                    .await;
            }
            ConnectionState::Config => {
                client
                    .send_packet_now(&CConfigDisconnect::new(&reason.get_text()))
                    .await;
            }
            // This way players get kicked when players using client functions (e.g. poll, send_packet)
            ConnectionState::Play => client.send_packet_now(&CPlayDisconnect::new(&reason)).await,
            _ => {}
        }
        log::debug!("Closing connection for {}", client.id);
        client.close();
    }

    pub async fn send_packet_now(&self, client: &Client, packet: Vec<u8>) {
        if let Err(err) = self
            .network_writer
            .lock()
            .await
            .write_packet(packet.into())
            .await
        {
            // It is expected that the packet will fail if we are closed
            if !client.closed.load(std::sync::atomic::Ordering::Relaxed) {
                log::warn!("Failed to send packet to client {}: {}", client.id, err);
                // We now need to close the connection to the client since the stream is in an
                // unknown state
                client.close();
            }
        }
    }

    pub fn write_packet<P: ClientPacket>(
        packet: &P,
        write: impl Write,
    ) -> Result<(), WritingError> {
        let mut write = write;
        write.write_var_int(&VarInt(P::PACKET_ID))?;
        packet.write_packet_data(write)
    }

    pub async fn write_packet_data(&self, packet_data: Bytes) -> Result<(), PacketEncodeError> {
        self.network_writer
            .lock()
            .await
            .write_packet(packet_data)
            .await
    }

    /// Handles an incoming packet, routing it to the appropriate handler based on the current connection state.
    ///
    /// This function takes a `RawPacket` and routes it to the corresponding handler based on the current connection state.
    /// It supports the following connection states:
    ///
    /// - **Handshake:** Handles handshake packets.
    /// - **Status:** Handles status request and ping packets.
    /// - **Login/Transfer:** Handles login and transfer packets.
    /// - **Config:** Handles configuration packets.
    ///
    /// For the `Play` state, an error is logged as it indicates an invalid state for packet processing.
    ///
    /// # Arguments
    ///
    /// * `server`: A reference to the `Server` instance.
    /// * `packet`: A mutable reference to the `RawPacket` to be processed.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the packet was read and handled successfully.
    ///
    /// # Errors
    ///
    /// Returns a `DeserializerError` if an error occurs during packet deserialization.
    pub async fn handle_packet(
        client: &Client,
        server: &Server,
        packet: &RawPacket,
    ) -> Result<(), ReadingError> {
        match client.connection_state.load() {
            pumpkin_protocol::ConnectionState::HandShake => {
                Self::handle_handshake_packet(client, packet).await
            }
            pumpkin_protocol::ConnectionState::Status => {
                Self::handle_status_packet(client, server, packet).await
            }
            // TODO: Check config if transfer is enabled
            pumpkin_protocol::ConnectionState::Login
            | pumpkin_protocol::ConnectionState::Transfer => {
                Self::handle_login_packet(client, server, packet).await
            }
            pumpkin_protocol::ConnectionState::Config => {
                Self::handle_config_packet(client, server, packet).await
            }
            pumpkin_protocol::ConnectionState::Play => {
                log::error!("Invalid Connection state {:?}", client.connection_state);
                Ok(())
            }
        }
    }

    async fn handle_handshake_packet(
        client: &Client,
        packet: &RawPacket,
    ) -> Result<(), ReadingError> {
        log::debug!("Handling handshake group");
        let payload = &packet.payload[..];
        match packet.id {
            0 => {
                client.handle_handshake(SHandShake::read(payload)?).await;
            }
            _ => {
                log::error!(
                    "Failed to handle java packet id {} in Handshake state",
                    packet.id
                );
            }
        }
        Ok(())
    }

    async fn handle_status_packet(
        client: &Client,
        server: &Server,
        packet: &RawPacket,
    ) -> Result<(), ReadingError> {
        log::debug!("Handling status group");
        let payload = &packet.payload[..];
        match packet.id {
            SStatusRequest::PACKET_ID => {
                client.handle_status_request(server).await;
            }
            SStatusPingRequest::PACKET_ID => {
                client
                    .handle_ping_request(SStatusPingRequest::read(payload)?)
                    .await;
            }
            _ => {
                log::error!(
                    "Failed to handle java client packet id {} in Status State",
                    packet.id
                );
            }
        }

        Ok(())
    }

    async fn handle_login_packet(
        client: &Client,
        server: &Server,
        packet: &RawPacket,
    ) -> Result<(), ReadingError> {
        log::debug!("Handling login group for id");
        let payload = &packet.payload[..];
        match packet.id {
            SLoginStart::PACKET_ID => {
                client
                    .handle_login_start(server, SLoginStart::read(payload)?)
                    .await;
            }
            SEncryptionResponse::PACKET_ID => {
                client
                    .handle_encryption_response(server, SEncryptionResponse::read(payload)?)
                    .await;
            }
            SLoginPluginResponse::PACKET_ID => {
                client
                    .handle_plugin_response(SLoginPluginResponse::read(payload)?)
                    .await;
            }
            SLoginAcknowledged::PACKET_ID => {
                client.handle_login_acknowledged(server).await;
            }
            SLoginCookieResponse::PACKET_ID => {
                client.handle_login_cookie_response(&SLoginCookieResponse::read(payload)?);
            }
            _ => {
                log::error!(
                    "Failed to handle java client packet id {} in Login State",
                    packet.id
                );
            }
        }
        Ok(())
    }

    async fn handle_config_packet(
        client: &Client,
        server: &Server,
        packet: &RawPacket,
    ) -> Result<(), ReadingError> {
        log::debug!("Handling config group");
        let payload = &packet.payload[..];
        match packet.id {
            SClientInformationConfig::PACKET_ID => {
                client
                    .handle_client_information_config(SClientInformationConfig::read(payload)?)
                    .await;
            }
            SPluginMessage::PACKET_ID => {
                client
                    .handle_plugin_message(SPluginMessage::read(payload)?)
                    .await;
            }
            SAcknowledgeFinishConfig::PACKET_ID => {
                client.handle_config_acknowledged(server).await;
            }
            SKnownPacks::PACKET_ID => {
                client
                    .handle_known_packs(server, SKnownPacks::read(payload)?)
                    .await;
            }
            SConfigCookieResponse::PACKET_ID => {
                client.handle_config_cookie_response(&SConfigCookieResponse::read(payload)?);
            }
            SConfigResourcePack::PACKET_ID => {
                client
                    .handle_resource_pack_response(SConfigResourcePack::read(payload)?)
                    .await;
            }
            _ => {
                log::error!(
                    "Failed to handle java client packet id {} in Config State",
                    packet.id
                );
            }
        }
        Ok(())
    }
}
