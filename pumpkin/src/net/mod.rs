use std::{
    net::SocketAddr,
    num::NonZeroU8,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicI32},
    },
};

use crate::{
    data::{banned_ip_data::BANNED_IP_LIST, banned_player_data::BANNED_PLAYER_LIST},
    entity::player::{ChatMode, Hand},
    server::Server,
};

use bytes::Bytes;
use crossbeam::atomic::AtomicCell;
use pumpkin_config::networking::compression::CompressionInfo;
use pumpkin_protocol::{
    ClientPacket, ConnectionState, Property, RawPacket, ServerPacket,
    client::{config::CConfigDisconnect, login::CLoginDisconnect, play::CPlayDisconnect},
    packet_decoder::{NetworkDecoder, PacketDecodeError},
    packet_encoder::NetworkEncoder,
    ser::{ReadingError, packet::Packet},
    server::{
        config::{
            SAcknowledgeFinishConfig, SClientInformationConfig, SConfigCookieResponse,
            SConfigResourcePack, SKnownPacks, SPluginMessage,
        },
        handshake::SHandShake,
        login::{
            SEncryptionResponse, SLoginAcknowledged, SLoginCookieResponse, SLoginPluginResponse,
            SLoginStart,
        },
        status::{SStatusPingRequest, SStatusRequest},
    },
};
use pumpkin_util::{ProfileAction, text::TextComponent};
use serde::Deserialize;
use sha1::Digest;
use sha2::Sha256;
use tokio::{
    io::{BufReader, BufWriter},
    net::tcp::OwnedWriteHalf,
    sync::{
        Notify,
        mpsc::{Receiver, Sender},
    },
    task::JoinHandle,
};
use tokio::{
    net::{TcpStream, tcp::OwnedReadHalf},
    sync::Mutex,
};

use thiserror::Error;
use tokio_util::task::TaskTracker;
use uuid::Uuid;
pub mod authentication;
mod container;
pub mod lan_broadcast;
mod packet;
mod proxy;
pub mod query;
pub mod rcon;

#[derive(Deserialize, Clone, Debug)]
pub struct GameProfile {
    pub id: Uuid,
    pub name: String,
    pub properties: Vec<Property>,
    #[serde(rename = "profileActions")]
    pub profile_actions: Option<Vec<ProfileAction>>,
}

pub fn offline_uuid(username: &str) -> Result<Uuid, uuid::Error> {
    Uuid::from_slice(&Sha256::digest(username)[..16])
}

/// Represents a player's configuration settings.
///
/// This struct contains various options that can be customized by the player, affecting their gameplay experience.
///
/// **Usage:**
///
/// This struct is typically used to store and manage a player's preferences. It can be sent to the server when a player joins or when they change their settings.
#[derive(Clone)]
pub struct PlayerConfig {
    /// The player's preferred language.
    pub locale: String, // 16
    /// The maximum distance at which chunks are rendered.
    pub view_distance: NonZeroU8,
    /// The player's chat mode settings
    pub chat_mode: ChatMode,
    /// Whether chat colors are enabled.
    pub chat_colors: bool,
    /// The player's skin configuration options.
    pub skin_parts: u8,
    /// The player's dominant hand (left or right).
    pub main_hand: Hand,
    /// Whether text filtering is enabled.
    pub text_filtering: bool,
    /// Whether the player wants to appear in the server list.
    pub server_listing: bool,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            locale: "en_us".to_string(),
            view_distance: NonZeroU8::new(10).unwrap(),
            chat_mode: ChatMode::Enabled,
            chat_colors: true,
            skin_parts: 0,
            main_hand: Hand::Right,
            text_filtering: false,
            server_listing: false,
        }
    }
}

pub enum PacketHandlerState {
    PacketReady,
    Stop,
}

/// Everything which makes a Connection with our Server is a `Client`.
/// Client will become Players when they reach the `Play` state
pub struct Client {
    /// The client id. This is good for coorelating a connection with a player
    /// Only used for logging purposes
    pub id: usize,
    /// The client's game profile information.
    pub gameprofile: Mutex<Option<GameProfile>>,
    /// The client's configuration settings, Optional
    pub config: Mutex<Option<PlayerConfig>>,
    /// The client's brand or modpack information, Optional.
    pub brand: Mutex<Option<String>>,
    /// The minecraft protocol version used by the client.
    pub protocol_version: AtomicI32,
    /// The Address used to connect to the Server, Send in the Handshake
    pub server_address: Mutex<String>,
    /// The current connection state of the client (e.g., Handshaking, Status, Play).
    pub connection_state: AtomicCell<ConnectionState>,
    /// Indicates if the client connection is closed.
    pub closed: Arc<AtomicBool>,
    /// The client's IP address.
    pub address: Mutex<SocketAddr>,
    /// The packet encoder for outgoing packets.
    network_writer: Arc<Mutex<NetworkEncoder<BufWriter<OwnedWriteHalf>>>>,
    /// The packet decoder for incoming packets.
    network_reader: Mutex<NetworkDecoder<BufReader<OwnedReadHalf>>>,
    /// Indicates whether the client should be converted into a player.
    pub make_player: AtomicBool,
    /// A collection of tasks associated with this client. The tasks await completion when removing the client.
    tasks: TaskTracker,
    /// An notifier that is triggered when this client is closed.
    close_interrupt: Arc<Notify>,
    /// A queue of serialized packets to send to the network
    outgoing_packet_queue_send: Sender<Bytes>,
    /// A queue of serialized packets to send to the network
    outgoing_packet_queue_recv: Option<Receiver<Bytes>>,
}

impl Client {
    #[must_use]
    pub fn new(tcp_stream: TcpStream, address: SocketAddr, id: usize) -> Self {
        let (read, write) = tcp_stream.into_split();
        let (send, recv) = tokio::sync::mpsc::channel(128);
        Self {
            id,
            protocol_version: AtomicI32::new(0),
            gameprofile: Mutex::new(None),
            config: Mutex::new(None),
            brand: Mutex::new(None),
            server_address: Mutex::new(String::new()),
            address: Mutex::new(address),
            connection_state: AtomicCell::new(ConnectionState::HandShake),
            network_writer: Arc::new(Mutex::new(NetworkEncoder::new(BufWriter::new(write)))),
            network_reader: Mutex::new(NetworkDecoder::new(BufReader::new(read))),
            closed: Arc::new(AtomicBool::new(false)),
            make_player: AtomicBool::new(false),
            close_interrupt: Arc::new(Notify::new()),
            tasks: TaskTracker::new(),
            outgoing_packet_queue_send: send,
            outgoing_packet_queue_recv: Some(recv),
        }
    }

    pub fn init(&mut self) {
        self.start_outgoing_packet_task();
    }

    fn start_outgoing_packet_task(&mut self) {
        let mut packet_receiver = self
            .outgoing_packet_queue_recv
            .take()
            .expect("This was set in the new fn");
        let writer = self.network_writer.clone();
        let close_interrupt = self.close_interrupt.clone();
        let closed = self.closed.clone();
        let id = self.id;
        self.spawn_task(async move {
            while !closed.load(std::sync::atomic::Ordering::Relaxed) {
                let recv_result = tokio::select! {
                    () = close_interrupt.notified() => {
                        None
                    },
                    recv_result = packet_receiver.recv() => {
                        recv_result
                    }
                };

                let Some(packet_data) = recv_result else {
                    break;
                };

                if let Err(err) = writer.lock().await.write_packet(packet_data).await {
                    // It is expected that the packet will fail if we are closed
                    if !closed.load(std::sync::atomic::Ordering::Relaxed) {
                        log::warn!("Failed to send packet to client {id}: {err}",);
                        // We now need to close the connection to the client since the stream is in an
                        // unknown state
                        Self::thread_safe_close(&close_interrupt, &closed);
                        break;
                    }
                }
            }
        });
    }

    pub async fn await_close_interrupt(&self) {
        self.close_interrupt.notified().await;
    }

    pub async fn await_tasks(&self) {
        self.tasks.close();
        self.tasks.wait().await;
    }

    /// Spawns a task associated with this client. All tasks spawned with this method are awaited
    /// when the client. This means tasks should complete in a reasonable amount of time or select
    /// on `Self::await_close_interrupt` to cancel the task when the client is closed
    ///
    /// Returns an `Option<JoinHandle<F::Output>>`. If the client is closed, this returns `None`.
    pub fn spawn_task<F>(&self, task: F) -> Option<JoinHandle<F::Output>>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        if self.closed.load(std::sync::atomic::Ordering::Relaxed) {
            None
        } else {
            Some(self.tasks.spawn(task))
        }
    }

    /// Enables packet encryption for the connection.
    ///
    /// This function takes a shared secret as input. The connection's encryption is enabled
    /// using the provided secret key.
    ///
    /// # Arguments
    ///
    /// * `shared_secret`: An **already decrypted** shared secret key used for encryption.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the encryption was set successfully.
    ///
    /// # Errors
    ///
    /// Returns an `EncryptionError` if the shared secret has an incorrect length.
    ///
    /// # Examples
    /// ```
    ///  let shared_secret = server.decrypt(&encryption_response.shared_secret).unwrap();
    ///
    ///  if let Err(error) = self.set_encryption(&shared_secret).await {
    ///       self.kick(&error.to_string()).await;
    ///       return;
    ///  }
    /// ```
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

    /// Enables packet compression for the connection.
    ///
    /// This function takes a `CompressionInfo` struct as input.
    /// packet compression is enabled with the specified threshold.
    ///
    /// # Arguments
    ///
    /// * `compression`: A `CompressionInfo` struct containing the compression threshold and compression level.
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

    /// Gets the next packet from the network or `None` if the connection has closed
    pub async fn get_packet(&self) -> Option<RawPacket> {
        let mut network_reader = self.network_reader.lock().await;
        tokio::select! {
            () = self.await_close_interrupt() => {
                log::debug!("Canceling player packet processing");
                None
            },
            packet_result = network_reader.get_raw_packet() => {
                match packet_result {
                    Ok(packet) => Some(packet),
                    Err(err) => {
                        if !matches!(err, PacketDecodeError::ConnectionClosed) {
                            log::warn!("Failed to decode packet from client {}: {}", self.id, err);
                            let text = format!("Error while reading incoming packet {err}");
                            self.kick(TextComponent::text(text)).await;
                        }
                        None
                    }
                }
            }
        }
    }

    /// Queues a clientbound packet to be sent to the connected client. Queued chunks are sent
    /// in-order to the client
    ///
    /// # Arguments
    ///
    /// * `packet`: A reference to a packet object implementing the `ClientPacket` trait.
    pub async fn enqueue_packet<P>(&self, packet: &P)
    where
        P: ClientPacket,
    {
        let mut packet_buf = Vec::new();
        if let Err(err) = packet.write(&mut packet_buf) {
            log::error!("Failed to serialize packet {}: {}", P::PACKET_ID, err);
            return;
        }
        self.enqueue_packet_data(packet_buf.into()).await;
    }

    pub async fn enqueue_packet_data(&self, packet_data: Bytes) {
        if let Err(err) = self.outgoing_packet_queue_send.send(packet_data).await {
            // This is expected to fail if we are closed
            if !self.closed.load(std::sync::atomic::Ordering::Relaxed) {
                log::error!(
                    "Failed to add packet to the outgoing packet queue for client {}: {}",
                    self.id,
                    err
                );
            }
        }
    }

    /// Sends a clientbound packet to the connected client and awaits until the packet is sent.
    /// Useful for blocking until the client has received a packet. Ignores the order of
    /// `enqueue_chunk`.
    ///
    /// # Arguments
    ///
    /// * `packet`: A reference to a packet object implementing the `ClientPacket` trait.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the Packet was Send successfully.
    ///
    /// # Errors
    ///
    /// Returns an `PacketError` if the packet could not be Send.
    pub async fn send_packet_now<P: ClientPacket>(&self, packet: &P) {
        let mut packet_buf = Vec::new();
        if let Err(err) = packet.write(&mut packet_buf) {
            log::error!("Failed to serialize packet {}: {}", P::PACKET_ID, err);
            return;
        }

        if let Err(err) = self
            .network_writer
            .lock()
            .await
            .write_packet(packet_buf.into())
            .await
        {
            // It is expected that the packet will fail if we are closed
            if !self.closed.load(std::sync::atomic::Ordering::Relaxed) {
                log::warn!("Failed to send packet to client {}: {}", self.id, err);
                // We now need to close the connection to the client since the stream is in an
                // unknown state
                self.close();
            }
        }
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
    pub async fn process_packets(&self, server: &Server) {
        while !self.make_player.load(std::sync::atomic::Ordering::Relaxed) {
            let packet = self.get_packet().await;
            let Some(packet) = packet else { break };

            if let Err(error) = self.handle_packet(server, &packet).await {
                let text = format!("Error while reading incoming packet {error}");
                log::error!(
                    "Failed to read incoming packet with id {}: {}",
                    packet.id,
                    error
                );
                self.kick(TextComponent::text(text)).await;
            }
        }
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
        &self,
        server: &Server,
        packet: &RawPacket,
    ) -> Result<(), ReadingError> {
        match self.connection_state.load() {
            pumpkin_protocol::ConnectionState::HandShake => {
                self.handle_handshake_packet(packet).await
            }
            pumpkin_protocol::ConnectionState::Status => {
                self.handle_status_packet(server, packet).await
            }
            // TODO: Check config if transfer is enabled
            pumpkin_protocol::ConnectionState::Login
            | pumpkin_protocol::ConnectionState::Transfer => {
                self.handle_login_packet(server, packet).await
            }
            pumpkin_protocol::ConnectionState::Config => {
                self.handle_config_packet(server, packet).await
            }
            pumpkin_protocol::ConnectionState::Play => {
                log::error!("Invalid Connection state {:?}", self.connection_state);
                Ok(())
            }
        }
    }

    async fn handle_handshake_packet(&self, packet: &RawPacket) -> Result<(), ReadingError> {
        log::debug!("Handling handshake group");
        let payload = &packet.payload[..];
        match packet.id {
            0 => {
                self.handle_handshake(SHandShake::read(payload)?).await;
            }
            _ => {
                log::error!(
                    "Failed to handle packet id {} in Handshake state",
                    packet.id
                );
            }
        }
        Ok(())
    }

    async fn handle_status_packet(
        &self,
        server: &Server,
        packet: &RawPacket,
    ) -> Result<(), ReadingError> {
        log::debug!("Handling status group");
        let payload = &packet.payload[..];
        match packet.id {
            SStatusRequest::PACKET_ID => {
                self.handle_status_request(server).await;
            }
            SStatusPingRequest::PACKET_ID => {
                self.handle_ping_request(SStatusPingRequest::read(payload)?)
                    .await;
            }
            _ => {
                log::error!(
                    "Failed to handle client packet id {} in Status State",
                    packet.id
                );
            }
        }

        Ok(())
    }

    async fn handle_login_packet(
        &self,
        server: &Server,
        packet: &RawPacket,
    ) -> Result<(), ReadingError> {
        log::debug!("Handling login group for id");
        let payload = &packet.payload[..];
        match packet.id {
            SLoginStart::PACKET_ID => {
                self.handle_login_start(server, SLoginStart::read(payload)?)
                    .await;
            }
            SEncryptionResponse::PACKET_ID => {
                self.handle_encryption_response(server, SEncryptionResponse::read(payload)?)
                    .await;
            }
            SLoginPluginResponse::PACKET_ID => {
                self.handle_plugin_response(SLoginPluginResponse::read(payload)?)
                    .await;
            }
            SLoginAcknowledged::PACKET_ID => {
                self.handle_login_acknowledged(server).await;
            }
            SLoginCookieResponse::PACKET_ID => {
                self.handle_login_cookie_response(&SLoginCookieResponse::read(payload)?);
            }
            _ => {
                log::error!(
                    "Failed to handle client packet id {} in Login State",
                    packet.id
                );
            }
        }
        Ok(())
    }

    async fn handle_config_packet(
        &self,
        server: &Server,
        packet: &RawPacket,
    ) -> Result<(), ReadingError> {
        log::debug!("Handling config group");
        let payload = &packet.payload[..];
        match packet.id {
            SClientInformationConfig::PACKET_ID => {
                self.handle_client_information_config(SClientInformationConfig::read(payload)?)
                    .await;
            }
            SPluginMessage::PACKET_ID => {
                self.handle_plugin_message(SPluginMessage::read(payload)?)
                    .await;
            }
            SAcknowledgeFinishConfig::PACKET_ID => {
                self.handle_config_acknowledged().await;
            }
            SKnownPacks::PACKET_ID => {
                self.handle_known_packs(server, SKnownPacks::read(payload)?)
                    .await;
            }
            SConfigCookieResponse::PACKET_ID => {
                self.handle_config_cookie_response(&SConfigCookieResponse::read(payload)?);
            }
            SConfigResourcePack::PACKET_ID => {
                self.handle_resource_pack_response(SConfigResourcePack::read(payload)?)
                    .await;
            }
            _ => {
                log::error!(
                    "Failed to handle client packet id {} in Config State",
                    packet.id
                );
            }
        }
        Ok(())
    }

    /// Disconnects a client from the server with a specified reason.
    ///
    /// This function kicks a client identified by its ID from the server. The appropriate disconnect packet is sent based on the client's current connection state.
    ///
    /// # Arguments
    ///
    /// * `reason`: A string describing the reason for kicking the client.
    pub async fn kick(&self, reason: TextComponent) {
        match self.connection_state.load() {
            ConnectionState::Login => {
                // TextComponent implements Serialze and writes in bytes instead of String, thats the reasib we only use content
                self.send_packet_now(&CLoginDisconnect::new(
                    &serde_json::to_string(&reason.0).unwrap_or_else(|_| String::new()),
                ))
                .await;
            }
            ConnectionState::Config => {
                self.send_packet_now(&CConfigDisconnect::new(&reason.get_text()))
                    .await;
            }
            // This way players get kicked when players using client functions (e.g. poll, send_packet)
            ConnectionState::Play => self.send_packet_now(&CPlayDisconnect::new(&reason)).await,
            _ => {
                log::warn!("Can't kick in {:?} State", self.connection_state);
                return;
            }
        }
        log::debug!("Closing connection for {}", self.id);
        self.close();
    }

    /// Checks if the client can join the server.
    pub async fn can_not_join(&self) -> Option<TextComponent> {
        let profile = self.gameprofile.lock().await;
        let Some(profile) = profile.as_ref() else {
            return Some(TextComponent::text("Missing GameProfile"));
        };

        let mut banned_players = BANNED_PLAYER_LIST.write().await;
        if let Some(entry) = banned_players.get_entry(profile) {
            let text = TextComponent::translate(
                "multiplayer.disconnect.banned.reason",
                [TextComponent::text(entry.reason.clone())],
            );
            return Some(match entry.expires {
                Some(expires) => text.add_child(TextComponent::translate(
                    "multiplayer.disconnect.banned.expiration",
                    [TextComponent::text(
                        expires.format("%F at %T %Z").to_string(),
                    )],
                )),
                None => text,
            });
        }
        drop(banned_players);

        let mut banned_ips = BANNED_IP_LIST.write().await;
        let address = self.address.lock().await;
        if let Some(entry) = banned_ips.get_entry(&address.ip()) {
            let text = TextComponent::translate(
                "multiplayer.disconnect.banned_ip.reason",
                [TextComponent::text(entry.reason.clone())],
            );
            return Some(match entry.expires {
                Some(expires) => text.add_child(TextComponent::translate(
                    "multiplayer.disconnect.banned_ip.expiration",
                    [TextComponent::text(
                        expires.format("%F at %T %Z").to_string(),
                    )],
                )),
                None => text,
            });
        }

        None
    }

    fn thread_safe_close(interrupt: &Arc<Notify>, closed: &Arc<AtomicBool>) {
        interrupt.notify_waiters();
        closed.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// Closes the connection to the client.
    ///
    /// This function marks the connection as closed using an atomic flag. It's generally preferable
    /// to use the `kick` function if you want to send a specific message to the client explaining the reason for the closure.
    /// However, use `close` in scenarios where sending a message is not critical or might not be possible (e.g., sudden connection drop).
    ///
    /// # Notes
    ///
    /// This function does not attempt to send any disconnect packets to the client.
    pub fn close(&self) {
        self.close_interrupt.notify_waiters();
        self.closed
            .store(true, std::sync::atomic::Ordering::Relaxed);
        log::debug!("Closed connection for {}", self.id);
    }
}

#[derive(Error, Debug)]
pub enum EncryptionError {
    #[error("failed to decrypt shared secret")]
    FailedDecrypt,
    #[error("shared secret has the wrong length")]
    SharedWrongLength,
}
