use std::net::SocketAddr;
use std::{io::Write, sync::Arc};

use bytes::Bytes;
use crossbeam::atomic::AtomicCell;
use pumpkin_config::networking::compression::CompressionInfo;
use pumpkin_data::packet::CURRENT_MC_PROTOCOL;
use pumpkin_protocol::java::server::play::{
    SChangeGameMode, SChatCommand, SChatMessage, SChunkBatch, SClickSlot, SClientCommand,
    SClientInformationPlay, SClientTickEnd, SCloseContainer, SCommandSuggestion, SConfirmTeleport,
    SCookieResponse as SPCookieResponse, SCustomPayload, SInteract, SKeepAlive, SPickItemFromBlock,
    SPlayPingRequest, SPlayerAbilities, SPlayerAction, SPlayerCommand, SPlayerInput, SPlayerLoaded,
    SPlayerPosition, SPlayerPositionRotation, SPlayerRotation, SPlayerSession, SSetCommandBlock,
    SSetCreativeSlot, SSetHeldItem, SSetPlayerGround, SSwingArm, SUpdateSign, SUseItem, SUseItemOn,
};
use pumpkin_protocol::packet::MultiVersionJavaPacket;
use pumpkin_protocol::{
    ClientPacket, ConnectionState, PacketDecodeError, RawPacket, ServerPacket,
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
    ser::{NetworkWriteExt, ReadingError, WritingError},
};
use pumpkin_util::text::TextComponent;
use pumpkin_util::version::MinecraftVersion;
use tokio::{
    io::{BufReader, BufWriter},
    net::{
        TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
    sync::Mutex,
};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

pub mod config;
pub mod handshake;
pub mod login;
pub mod play;
pub mod status;

use crate::entity::player::Player;
use crate::net::{GameProfile, PlayerConfig};
use crate::plugin::player::player_custom_payload::PlayerCustomPayloadEvent;
use crate::{error::PumpkinError, net::EncryptionError, server::Server};

pub struct JavaClient {
    pub id: u64,
    pub version: AtomicCell<MinecraftVersion>,
    /// The client's game profile information.
    pub gameprofile: Mutex<Option<GameProfile>>,
    /// The client's configuration settings, Optional
    pub config: Mutex<Option<PlayerConfig>>,
    /// The Address used to connect to the Server, Send in the Handshake
    pub server_address: Mutex<String>,
    /// The current connection state of the client (e.g., Handshaking, Status, Play).
    pub connection_state: AtomicCell<ConnectionState>,
    /// The client's IP address.
    pub address: Mutex<SocketAddr>,
    /// The client's brand or modpack information, Optional.
    pub brand: Mutex<Option<String>>,
    /// A collection of tasks associated with this client. The tasks await completion when removing the client.
    tasks: TaskTracker,
    /// An notifier that is triggered when this client is closed.
    close_token: CancellationToken,
    /// A queue of serialized packets to send to the network
    outgoing_packet_queue_send: Sender<Bytes>,
    /// A queue of serialized packets to send to the network
    outgoing_packet_queue_recv: Option<Receiver<Bytes>>,
    /// The packet encoder for outgoing packets.
    network_writer: Arc<Mutex<TCPNetworkEncoder<BufWriter<OwnedWriteHalf>>>>,
    /// The packet decoder for incoming packets.
    network_reader: Mutex<TCPNetworkDecoder<BufReader<OwnedReadHalf>>>,
}

pub enum PacketHandlerResult {
    Stop,
    // Signal to spawn the player
    ReadyToPlay(GameProfile, PlayerConfig),
}

impl JavaClient {
    #[must_use]
    pub fn new(tcp_stream: TcpStream, address: SocketAddr, id: u64) -> Self {
        let (read, write) = tcp_stream.into_split();
        let (send, recv) = tokio::sync::mpsc::channel(128);
        Self {
            id,
            gameprofile: Mutex::new(None),
            config: Mutex::new(None),
            server_address: Mutex::new(String::new()),
            address: Mutex::new(address),
            connection_state: AtomicCell::new(ConnectionState::HandShake),
            close_token: CancellationToken::new(),
            tasks: TaskTracker::new(),
            outgoing_packet_queue_send: send,
            outgoing_packet_queue_recv: Some(recv),
            version: AtomicCell::new(MinecraftVersion::from_protocol(CURRENT_MC_PROTOCOL)),
            network_writer: Arc::new(Mutex::new(TCPNetworkEncoder::new(BufWriter::new(write)))),
            network_reader: Mutex::new(TCPNetworkDecoder::new(BufReader::new(read))),
            brand: Mutex::new(None),
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
    pub async fn handle_login_sequence(&self, server: &Arc<Server>) -> PacketHandlerResult {
        while let Some(packet) = self.get_packet().await {
            match self.handle_packet(server, &packet).await {
                Ok(result) => {
                    if let Some(result) = result {
                        return result;
                    }
                }
                Err(error) => {
                    let text = format!("Error while reading incoming packet {error}");
                    log::debug!(
                        "Failed to read incoming packet with id {}: {}",
                        packet.id,
                        error
                    );
                    self.kick(TextComponent::text(text)).await;
                }
            }
        }
        PacketHandlerResult::Stop
    }

    pub async fn progress_player_packets(&self, player: &Arc<Player>, server: &Arc<Server>) {
        while let Some(packet) = self.get_packet().await {
            match self.handle_play_packet(player, server, &packet).await {
                Ok(()) => {}
                Err(e) => {
                    if e.is_kick() {
                        if let Some(kick_reason) = e.client_kick_reason() {
                            self.kick(TextComponent::text(kick_reason)).await;
                        } else {
                            self.kick(TextComponent::text(format!(
                                "Error while handling incoming packet {e}"
                            )))
                            .await;
                        }
                    }
                    e.log();
                }
            }
        }
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
        if self.close_token.is_cancelled() {
            None
        } else {
            Some(self.tasks.spawn(task))
        }
    }

    pub async fn enqueue_packet<P: ClientPacket>(&self, packet: &P) {
        let mut buf = Vec::new();
        let writer = &mut buf;
        self.write_packet(packet, writer).unwrap();
        self.enqueue_packet_data(buf.into()).await;
    }

    /// Queues a clientbound packet to be sent to the connected client. Queued chunks are sent
    /// in-order to the client
    ///
    /// # Arguments
    ///
    /// * `packet`: A reference to a packet object implementing the `ClientPacket` trait.
    pub async fn enqueue_packet_data(&self, packet_data: Bytes) {
        if let Err(err) = self.outgoing_packet_queue_send.send(packet_data).await {
            // This is expected to fail if we are closed
            if !self.close_token.is_cancelled() {
                log::error!(
                    "Failed to add packet to the outgoing packet queue for client {}: {}",
                    self.id,
                    err
                );
            }
        }
    }

    pub async fn await_close_interrupt(&self) {
        self.close_token.cancelled().await;
    }

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

    pub async fn kick(&self, reason: TextComponent) {
        match self.connection_state.load() {
            ConnectionState::Login => {
                // TextComponent implements Serialize and writes in bytes instead of String, that's the reasib we only use content
                self.send_packet_now(&CLoginDisconnect::new(
                    serde_json::to_string(&reason.0).unwrap_or_else(|_| String::new()),
                ))
                .await;
            }
            ConnectionState::Config => {
                self.send_packet_now(&CConfigDisconnect::new(&reason.get_text()))
                    .await;
            }
            ConnectionState::Play => self.send_packet_now(&CPlayDisconnect::new(&reason)).await,
            _ => {}
        }
        log::debug!("Closing connection for {}", self.id);
        self.close();
    }

    pub async fn send_packet_now<P: ClientPacket>(&self, packet: &P) {
        let mut packet_buf = Vec::new();
        let writer = &mut packet_buf;
        self.write_packet(packet, writer).unwrap();
        self.send_packet_now_data(packet_buf).await;
    }

    pub async fn send_packet_now_data(&self, packet: Vec<u8>) {
        if let Err(err) = self
            .network_writer
            .lock()
            .await
            .write_packet(packet.into())
            .await
        {
            // It is expected that the packet will fail if we are closed
            if !self.close_token.is_cancelled() {
                log::warn!("Failed to send packet to client {}: {}", self.id, err);
                // We now need to close the connection to the client since the stream is in an
                // unknown state
                self.close();
            }
        }
    }

    pub fn write_packet<P: ClientPacket>(
        &self,
        packet: &P,
        write: impl Write,
    ) -> Result<(), WritingError> {
        let mut write = write;
        let version = self.version.load();
        write.write_var_int(&VarInt(P::PACKET_ID.to_id(version)))?;
        packet.write_packet_data(write, &version)
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
        server: &Arc<Server>,
        packet: &RawPacket,
    ) -> Result<Option<PacketHandlerResult>, ReadingError> {
        match self.connection_state.load() {
            ConnectionState::HandShake => self.handle_handshake_packet(packet).await,
            ConnectionState::Status => self.handle_status_packet(server, packet).await,
            // TODO: Check config if transfer is enabled
            ConnectionState::Login | ConnectionState::Transfer => {
                self.handle_login_packet(server, packet).await
            }
            ConnectionState::Config => self.handle_config_packet(server, packet).await,
            ConnectionState::Play => Ok(None),
        }
    }

    async fn handle_handshake_packet(
        &self,
        packet: &RawPacket,
    ) -> Result<Option<PacketHandlerResult>, ReadingError> {
        log::debug!("Handling handshake group");
        let payload = &packet.payload[..];
        match packet.id {
            0 => {
                self.handle_handshake(SHandShake::read(payload)?).await;
                Ok(None)
            }
            _ => Err(ReadingError::Message(format!(
                "Failed to handle packet id {} in Handshake State",
                packet.id
            ))),
        }
    }

    async fn handle_status_packet(
        &self,
        server: &Server,
        packet: &RawPacket,
    ) -> Result<Option<PacketHandlerResult>, ReadingError> {
        log::debug!("Handling status group");
        let payload = &packet.payload[..];
        match packet.id {
            id if id == SStatusRequest::PACKET_ID => {
                self.handle_status_request(server).await;
                Ok(None)
            }
            id if id == SStatusPingRequest::PACKET_ID => {
                self.handle_ping_request(SStatusPingRequest::read(payload)?)
                    .await;
                Ok(None)
            }
            _ => Err(ReadingError::Message(format!(
                "Failed to handle java client packet id {} in Status State",
                packet.id
            ))),
        }
    }

    pub fn start_outgoing_packet_task(&mut self) {
        let mut packet_receiver = self
            .outgoing_packet_queue_recv
            .take()
            .expect("This was set in the new fn");
        let close_token = self.close_token.clone();
        let writer = self.network_writer.clone();
        let id = self.id;
        self.spawn_task(async move {
            while !close_token.is_cancelled() {
                let recv_result = tokio::select! {
                    () =  close_token.cancelled() => None,
                    res = packet_receiver.recv() => res,
                };

                let Some(packet_data) = recv_result else {
                    break;
                };

                if let Err(err) = writer.lock().await.write_packet(packet_data).await {
                    // It is expected that the packet will fail if we are closed
                    if !close_token.is_cancelled() {
                        log::warn!("Failed to send packet to client {id}: {err}",);
                        // We now need to close the connection to the client since the stream is in an
                        // unknown state
                        close_token.cancel();
                        break;
                    }
                }
            }
        });
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
        self.close_token.cancel();
    }

    pub fn is_closed(&self) -> bool {
        self.close_token.is_cancelled()
    }

    async fn handle_login_packet(
        &self,
        server: &Server,
        packet: &RawPacket,
    ) -> Result<Option<PacketHandlerResult>, ReadingError> {
        log::debug!("Handling login group for id");
        let payload = &packet.payload[..];
        match packet.id {
            id if id == SLoginStart::PACKET_ID => {
                self.handle_login_start(server, SLoginStart::read(payload)?)
                    .await;
            }
            id if id == SEncryptionResponse::PACKET_ID => {
                self.handle_encryption_response(server, SEncryptionResponse::read(payload)?)
                    .await;
            }
            id if id == SLoginPluginResponse::PACKET_ID => {
                self.handle_plugin_response(server, SLoginPluginResponse::read(payload)?)
                    .await;
            }
            id if id == SLoginAcknowledged::PACKET_ID => {
                self.handle_login_acknowledged(server).await;
            }
            id if id == SLoginCookieResponse::PACKET_ID => {
                self.handle_login_cookie_response(&SLoginCookieResponse::read(payload)?);
            }
            _ => {
                log::error!(
                    "Failed to handle java client packet id {} in Login State",
                    packet.id
                );
            }
        }
        Ok(None)
    }

    async fn handle_config_packet(
        &self,
        server: &Arc<Server>,
        packet: &RawPacket,
    ) -> Result<Option<PacketHandlerResult>, ReadingError> {
        log::debug!("Handling config group for id {}", packet.id);
        let payload = &packet.payload[..];

        match packet.id {
            id if id == SClientInformationConfig::PACKET_ID => {
                self.handle_client_information_config(SClientInformationConfig::read(payload)?)
                    .await;
            }
            id if id == SPluginMessage::PACKET_ID => {
                self.handle_plugin_message(SPluginMessage::read(payload)?)
                    .await;
            }
            id if id == SAcknowledgeFinishConfig::PACKET_ID => {
                return Ok(Some(self.handle_config_acknowledged(server).await));
            }
            id if id == SKnownPacks::PACKET_ID => {
                self.handle_known_packs(SKnownPacks::read(payload)?).await;
            }
            id if id == SConfigCookieResponse::PACKET_ID => {
                self.handle_config_cookie_response(&SConfigCookieResponse::read(payload)?);
            }
            id if id == SConfigResourcePack::PACKET_ID => {
                self.handle_resource_pack_response(server, SConfigResourcePack::read(payload)?)
                    .await;
            }
            _ => {
                log::error!(
                    "Failed to handle java client packet id {} in Config State",
                    packet.id
                );
            }
        }
        Ok(None)
    }

    #[expect(clippy::too_many_lines)]
    pub async fn handle_play_packet(
        &self,
        player: &Arc<Player>,
        server: &Arc<Server>,
        packet: &RawPacket,
    ) -> Result<(), Box<dyn PumpkinError>> {
        let payload = &packet.payload[..];

        match packet.id {
            id if id == SConfirmTeleport::PACKET_ID => {
                self.handle_confirm_teleport(player, SConfirmTeleport::read(payload)?)
                    .await;
            }
            id if id == SChangeGameMode::PACKET_ID => {
                self.handle_change_game_mode(player, SChangeGameMode::read(payload)?)
                    .await;
            }
            id if id == SChatCommand::PACKET_ID => {
                self.handle_chat_command(player, server, &(SChatCommand::read(payload)?))
                    .await;
            }
            id if id == SChatMessage::PACKET_ID => {
                self.handle_chat_message(server, player, SChatMessage::read(payload)?)
                    .await;
            }
            id if id == SClientInformationPlay::PACKET_ID => {
                self.handle_client_information(player, SClientInformationPlay::read(payload)?)
                    .await;
            }
            id if id == SClientCommand::PACKET_ID => {
                self.handle_client_status(player, SClientCommand::read(payload)?)
                    .await;
            }
            id if id == SPlayerInput::PACKET_ID => {
                self.handle_player_input(player, SPlayerInput::read(payload)?)
                    .await;
            }
            id if id == SInteract::PACKET_ID => {
                self.handle_interact(player, SInteract::read(payload)?, server)
                    .await;
            }
            id if id == SKeepAlive::PACKET_ID => {
                self.handle_keep_alive(player, SKeepAlive::read(payload)?)
                    .await;
            }
            id if id == SClientTickEnd::PACKET_ID => {
                // TODO
            }
            id if id == SPlayerPosition::PACKET_ID => {
                self.handle_position(player, server, SPlayerPosition::read(payload)?)
                    .await;
            }
            id if id == SPlayerPositionRotation::PACKET_ID => {
                self.handle_position_rotation(
                    player,
                    server,
                    SPlayerPositionRotation::read(payload)?,
                )
                .await;
            }
            id if id == SPlayerRotation::PACKET_ID => {
                self.handle_rotation(player, SPlayerRotation::read(payload)?)
                    .await;
            }
            id if id == SSetPlayerGround::PACKET_ID => {
                self.handle_player_ground(player, &SSetPlayerGround::read(payload)?);
            }
            id if id == SPickItemFromBlock::PACKET_ID => {
                self.handle_pick_item_from_block(player, SPickItemFromBlock::read(payload)?)
                    .await;
            }
            id if id == SPlayerAbilities::PACKET_ID => {
                self.handle_player_abilities(player, SPlayerAbilities::read(payload)?)
                    .await;
            }
            id if id == SPlayerAction::PACKET_ID => {
                self.handle_player_action(player, SPlayerAction::read(payload)?, server)
                    .await;
            }
            id if id == SSetCommandBlock::PACKET_ID => {
                self.handle_set_command_block(player, SSetCommandBlock::read(payload)?)
                    .await;
            }
            id if id == SPlayerCommand::PACKET_ID => {
                self.handle_player_command(player, SPlayerCommand::read(payload)?)
                    .await;
            }
            id if id == SPlayerLoaded::PACKET_ID => Self::handle_player_loaded(player),
            id if id == SPlayPingRequest::PACKET_ID => {
                self.handle_play_ping_request(SPlayPingRequest::read(payload)?)
                    .await;
            }
            id if id == SClickSlot::PACKET_ID => {
                player.on_slot_click(SClickSlot::read(payload)?).await;
            }
            id if id == SSetHeldItem::PACKET_ID => {
                self.handle_set_held_item(player, SSetHeldItem::read(payload)?)
                    .await;
            }
            id if id == SSetCreativeSlot::PACKET_ID => {
                self.handle_set_creative_slot(player, SSetCreativeSlot::read(payload)?)
                    .await?;
            }
            id if id == SSwingArm::PACKET_ID => {
                self.handle_swing_arm(player, SSwingArm::read(payload)?)
                    .await;
            }
            id if id == SUpdateSign::PACKET_ID => {
                self.handle_sign_update(player, SUpdateSign::read(payload)?)
                    .await;
            }
            id if id == SUseItemOn::PACKET_ID => {
                self.handle_use_item_on(player, SUseItemOn::read(payload)?, server)
                    .await?;
            }
            id if id == SUseItem::PACKET_ID => {
                self.handle_use_item(player, &SUseItem::read(payload)?, server)
                    .await;
            }
            id if id == SCommandSuggestion::PACKET_ID => {
                self.handle_command_suggestion(player, SCommandSuggestion::read(payload)?, server)
                    .await;
            }
            id if id == SPCookieResponse::PACKET_ID => {
                self.handle_cookie_response(&SPCookieResponse::read(payload)?);
            }
            id if id == SCloseContainer::PACKET_ID => {
                self.handle_close_container(player, server, SCloseContainer::read(payload)?)
                    .await;
            }
            id if id == SChunkBatch::PACKET_ID => {
                self.handle_chunk_batch(player, SChunkBatch::read(payload)?)
                    .await;
            }
            id if id == SPlayerSession::PACKET_ID => {
                self.handle_chat_session_update(player, server, SPlayerSession::read(payload)?)
                    .await;
            }
            id if id == SCustomPayload::PACKET_ID => {
                let payload = SCustomPayload::read(payload)?;
                let event = PlayerCustomPayloadEvent::new(
                    player.clone(),
                    payload.channel,
                    Bytes::from(payload.data),
                );
                server.plugin_manager.fire(event).await;
            }
            _ => {
                log::warn!("Failed to handle player packet id {}", packet.id);
            }
        }
        Ok(())
    }
}
