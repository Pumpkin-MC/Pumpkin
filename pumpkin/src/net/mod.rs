use std::{
    io::Write,
    net::SocketAddr,
    num::NonZeroU8,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicI32, Ordering},
    },
};

use crate::{
    data::{
        banned_ip_data::BANNED_IP_LIST, banned_player_data::BANNED_PLAYER_LIST,
        op_data::OPERATOR_CONFIG, whitelist_data::WHITELIST_CONFIG,
    },
    entity::player::{ChatMode, Hand},
    net::{bedrock::BedrockClientPlatform, java::JavaClientPlatform},
    server::Server,
};

use bytes::Bytes;
use crossbeam::atomic::AtomicCell;
use pumpkin_config::networking::compression::CompressionInfo;
use pumpkin_protocol::{
    ClientPacket, ConnectionState, PacketEncodeError, Property, RawPacket, ser::WritingError,
};
use pumpkin_util::{ProfileAction, text::TextComponent};
use serde::Deserialize;
use sha1::Digest;
use sha2::Sha256;
use tokio::sync::Mutex;
use tokio::{
    sync::{
        Notify,
        mpsc::{Receiver, Sender},
    },
    task::JoinHandle,
};

use thiserror::Error;
use tokio_util::task::TaskTracker;
use uuid::Uuid;
pub mod authentication;
pub mod bedrock;
pub mod java;
pub mod lan_broadcast;
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
    pub id: u64,
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
    /// Indicates if the client is added to the server listing.
    pub added_to_server_listing: AtomicBool,
    /// Indicates whether the client should be converted into a player.
    pub make_player: AtomicBool,
    pub platform: Arc<ClientPlatform>,
    /// A collection of tasks associated with this client. The tasks await completion when removing the client.
    tasks: TaskTracker,
    /// An notifier that is triggered when this client is closed.
    close_interrupt: Arc<Notify>,
    /// A queue of serialized packets to send to the network
    outgoing_packet_queue_send: Sender<Bytes>,
    /// A queue of serialized packets to send to the network
    outgoing_packet_queue_recv: Option<Receiver<Bytes>>,
}

pub enum ClientPlatform {
    Java(JavaClientPlatform),
    Bedrock(BedrockClientPlatform),
}

impl ClientPlatform {
    pub async fn write_packet_data(&self, packet_data: Bytes) -> Result<(), PacketEncodeError> {
        match self {
            Self::Java(java) => java.write_packet_data(packet_data).await,
            Self::Bedrock(bedrock) => bedrock.write_packet_data(packet_data).await,
        }
    }

    pub fn write_packet<P: ClientPacket>(
        &self,
        packet: &P,
        write: impl Write,
    ) -> Result<(), WritingError> {
        match self {
            Self::Java(_) => JavaClientPlatform::write_packet(packet, write),
            Self::Bedrock(_) => BedrockClientPlatform::write_packet(packet, write),
        }
    }

    pub async fn send_packet_now(&self, client: &Client, packet: Vec<u8>) {
        match self {
            Self::Java(java) => java.send_packet_now(client, packet).await,
            Self::Bedrock(bedrock) => bedrock.send_packet_now(client, packet).await,
        }
    }

    pub async fn kick(&self, client: &Client, reason: TextComponent) {
        match self {
            Self::Java(java) => java.kick(client, reason).await,
            Self::Bedrock(_bedrock) => todo!(),
        }
    }

    pub async fn set_encryption(
        &self,
        shared_secret: &[u8], // decrypted
    ) -> Result<(), EncryptionError> {
        match self {
            Self::Java(java) => java.set_encryption(shared_secret).await,
            Self::Bedrock(_bedrock) => todo!(),
        }
    }

    pub async fn set_compression(&self, compression: CompressionInfo) {
        match self {
            Self::Java(java) => java.set_compression(compression).await,
            Self::Bedrock(_bedrock_client_platform) => todo!(),
        }
    }

    pub async fn get_packet(&self, client: &Client) -> Option<RawPacket> {
        match self {
            Self::Java(java) => java.get_packet(client).await,
            Self::Bedrock(_bedrock) => todo!(),
        }
    }
}

impl Client {
    #[must_use]
    pub fn new(platform: ClientPlatform, address: SocketAddr, id: u64) -> Self {
        let (send, recv) = tokio::sync::mpsc::channel(128);
        Self {
            id,
            protocol_version: AtomicI32::new(0),
            gameprofile: Mutex::new(None),
            config: Mutex::new(None),
            brand: Mutex::new(None),
            server_address: Mutex::new(String::new()),
            address: Mutex::new(address),
            platform: Arc::new(platform),
            connection_state: AtomicCell::new(ConnectionState::HandShake),
            closed: Arc::new(AtomicBool::new(false)),
            make_player: AtomicBool::new(false),
            close_interrupt: Arc::new(Notify::new()),
            tasks: TaskTracker::new(),
            outgoing_packet_queue_send: send,
            outgoing_packet_queue_recv: Some(recv),
            added_to_server_listing: AtomicBool::new(false),
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
        let close_interrupt = self.close_interrupt.clone();
        let closed = self.closed.clone();
        let platform = self.platform.clone();
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

                if let Err(err) = platform.write_packet_data(packet_data).await {
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
        self.platform.set_encryption(shared_secret).await
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
        self.platform.set_compression(compression).await;
    }

    /// Gets the next packet from the network or `None` if the connection has closed
    pub async fn get_packet(&self) -> Option<RawPacket> {
        self.platform.get_packet(self).await
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
        let mut buf = Vec::new();
        let writer = &mut buf;
        self.platform.write_packet(packet, writer).unwrap();
        self.enqueue_packet_data(buf.into()).await;
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
        let writer = &mut packet_buf;
        self.platform.write_packet(packet, writer).unwrap();
        self.platform.send_packet_now(self, packet_buf).await;
    }

    /// Disconnects a client from the server with a specified reason.
    ///
    /// This function kicks a client identified by its ID from the server. The appropriate disconnect packet is sent based on the client's current connection state.
    ///
    /// # Arguments
    ///
    /// * `reason`: A string describing the reason for kicking the client.
    pub async fn kick(&self, reason: TextComponent) {
        self.platform.kick(self, reason).await;
    }

    /// Checks if the client can join the server.
    pub async fn can_not_join(&self, server: &Server) -> Option<TextComponent> {
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

        if server.white_list.load(Ordering::Relaxed) {
            let ops = OPERATOR_CONFIG.read().await;
            let whitelist = WHITELIST_CONFIG.read().await;

            if ops.get_entry(&profile.id).is_none() && !whitelist.is_whitelisted(profile) {
                return Some(TextComponent::translate(
                    "multiplayer.disconnect.not_whitelisted",
                    &[],
                ));
            }
        }

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

fn is_valid_player_name(name: &str) -> bool {
    name.len() <= 16 && name.chars().all(|c| c > 32u8 as char && c < 127u8 as char)
}
