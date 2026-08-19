use pumpkin_protocol::java::client::play::{
    CAcknowledgeBlockChange, CChunkBatchEnd, CChunkBatchStart, CChunkData, CPlayDisconnect,
};
use pumpkin_world::level::SyncChunk;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use std::{io::Write, sync::Arc};

use bytes::Bytes;
use crossbeam::atomic::AtomicCell;
use pumpkin_data::translation;
use pumpkin_protocol::java::server::play::{
    SAttack, SBlockEntityTagQuery, SBundleItemSelected, SChangeGameMode, SChatCommand,
    SChatMessage, SChunkBatch, SClickSlot, SClientCommand, SClientInformationPlay, SClientTickEnd,
    SCloseContainer, SCommandSuggestion, SConfigurationAcknowledged, SConfirmTeleport,
    SContainerButtonClick, SContainerSlotStateChanged, SCookieResponse as SPCookieResponse,
    SCustomPayload, SDebugSampleSubscription, SDebugSubscriptionRequest, SEditBook,
    SEntityTagQuery, SInteract, SJigsawGenerate, SLockDifficulty, SMoveVehicle, SPaddleBoat,
    SPickItemFromBlock, SPlaceRecipe, SPlayPingRequest, SPlayPong, SPlayResourcePack,
    SPlayerAbilities, SPlayerAction, SPlayerCommand, SPlayerInput, SPlayerLoaded, SPlayerPosition,
    SPlayerPositionRotation, SPlayerRotation, SPlayerSession, SRecipeBookChangeSettings,
    SRecipeBookSeenRecipe, SRenameItem, SSeenAdvancement, SSelectTrade, SSetCommandBlock,
    SSetCommandMinecart, SSetCreativeSlot, SSetGameRule, SSetHeldItem, SSetJigsawBlock,
    SSetPlayerGround, SSetStructureBlock, SSetTestBlock, SSpectateEntity, SSwingArm,
    STeleportToEntity, STestInstanceBlockAction, SUpdateSign, SUseItem, SUseItemOn,
};
use pumpkin_protocol::packet::MultiVersionJavaPacket;
use pumpkin_protocol::{
    ClientPacket, ConnectionState, PacketDecodeError, RawPacket, ServerPacket,
    codec::var_int::VarInt,
    java::{
        client::{config::CConfigDisconnect, login::CLoginDisconnect},
        packet_decoder::TCPNetworkDecoder,
        packet_encoder::TCPNetworkEncoder,
    },
    ser::{NetworkWriteExt, WritingError},
};
use pumpkin_util::text::TextComponent;
use pumpkin_util::version::JavaMinecraftVersion;
use tokio::{
    io::{BufReader, BufWriter},
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
    sync::oneshot,
};
use tokio::{
    sync::mpsc::{Receiver, Sender, error::TryRecvError},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::{debug, error, warn};

pub mod config;
pub mod handshake;
pub mod login;
pub mod pending;
pub mod play;
pub mod recipe_helper;
pub mod status;

use arc_swap::ArcSwap;
use pending::PendingConnection;

use crate::entity::player::Player;
use crate::net::{GameProfile, PacketHandlerResult, PacketRateLimiter, PlayerConfig};
use crate::plugin::api::events::world::chunk_send::ChunkSend;
use crate::plugin::player::player_custom_payload::PlayerCustomPayloadEvent;
use crate::{error::PumpkinError, server::Server};

pub struct JavaClient {
    pub id: u64,
    pub version: AtomicCell<JavaMinecraftVersion>,
    /// The client's game profile information. Direct field (lock-free).
    pub gameprofile: GameProfile,
    /// The client's configuration settings. Lock-free `ArcSwap`.
    pub config: ArcSwap<PlayerConfig>,
    /// The Address used to connect to the Server, Sent in the Handshake. Direct field.
    pub server_address: String,
    /// The current connection state of the client (e.g., Handshaking, Status, Play).
    pub connection_state: AtomicCell<ConnectionState>,
    /// The client's IP address. Direct field (lock-free).
    pub address: SocketAddr,
    /// The client's brand or modpack information. Lock-free `ArcSwap`.
    pub brand: ArcSwap<Option<String>>,
    /// Associated player reference. Lock-free `ArcSwap`.
    pub player: ArcSwap<Option<Arc<Player>>>,
    /// A collection of tasks associated with this client. The tasks await completion when removing the client.
    tasks: TaskTracker,
    /// An notifier that is triggered when this client is closed.
    close_token: CancellationToken,
    /// A normal-priority queue of serialized packets to send to the network.
    outgoing_packet_queue_send: Sender<OutgoingPacket>,
    /// A normal-priority queue of serialized packets to send to the network.
    outgoing_packet_queue_recv: Option<Receiver<OutgoingPacket>>,
    /// A high-priority queue of serialized packets to send to the network.
    outgoing_packet_priority_send: Sender<OutgoingPacket>,
    /// A high-priority queue of serialized packets to send to the network.
    outgoing_packet_priority_recv: Option<Receiver<OutgoingPacket>>,
    /// The packet encoder for outgoing packets.
    network_writer: std::sync::Mutex<Option<TCPNetworkEncoder<BufWriter<OwnedWriteHalf>>>>,
    /// The packet decoder for incoming packets.
    network_reader: std::sync::Mutex<Option<TCPNetworkDecoder<BufReader<OwnedReadHalf>>>>,
    /// Keep Alive:
    ///
    /// Whether we are waiting for a response after sending a keep alive packet.
    pub wait_for_keep_alive: AtomicBool,
    /// The keep alive packet payload we send. The client should respond with the same id.
    pub keep_alive_id: AtomicCell<i64>,
    /// The last time we sent a keep alive packet.
    pub last_keep_alive_time: AtomicCell<Instant>,
    /// The last time any packet was received from the client.
    pub last_packet_time: AtomicCell<Instant>,
    /// Recent in-flight keep alive IDs with their sent timestamps.
    pub pending_keep_alives: std::sync::Mutex<Vec<(i64, Instant)>>,

    pub packet_sequence: AtomicI32,
    /// Packet rate limiter for incoming client packets.
    pub packet_limiter: PacketRateLimiter,
}

pub enum OutgoingPacketType {
    Normal,
    HighPriority,
}

struct OutgoingPacket {
    data: Bytes,
    completion: Option<oneshot::Sender<()>>,
}

impl OutgoingPacket {
    const fn normal(data: Bytes) -> Self {
        Self {
            data,
            completion: None,
        }
    }

    const fn high_priority(data: Bytes, completion: oneshot::Sender<()>) -> Self {
        Self {
            data,
            completion: Some(completion),
        }
    }
}

impl JavaClient {
    #[must_use]
    pub fn from_pending(
        pending: PendingConnection,
        gameprofile: GameProfile,
        config: PlayerConfig,
    ) -> Self {
        let (send, recv) = tokio::sync::mpsc::channel(4096);
        let (priority_send, priority_recv) = tokio::sync::mpsc::channel(4096);

        Self {
            id: pending.id,
            gameprofile,
            config: ArcSwap::from_pointee(config),
            server_address: pending.server_address,
            address: pending.address,
            connection_state: pending.connection_state,
            close_token: pending.close_token,
            tasks: TaskTracker::new(),
            outgoing_packet_queue_send: send,
            outgoing_packet_queue_recv: Some(recv),
            outgoing_packet_priority_send: priority_send,
            outgoing_packet_priority_recv: Some(priority_recv),
            version: pending.version,
            network_writer: std::sync::Mutex::new(Some(pending.network_writer)),
            network_reader: std::sync::Mutex::new(Some(pending.network_reader)),
            brand: ArcSwap::from_pointee(pending.brand),
            player: ArcSwap::from_pointee(None),
            wait_for_keep_alive: AtomicBool::new(false),
            keep_alive_id: AtomicCell::new(0),
            last_keep_alive_time: AtomicCell::new(Instant::now()),
            last_packet_time: AtomicCell::new(Instant::now()),
            pending_keep_alives: std::sync::Mutex::new(Vec::new()),
            packet_sequence: AtomicI32::new(-1),
            packet_limiter: pending.packet_limiter,
        }
    }

    pub fn set_player(&self, player: Arc<Player>) {
        self.player.store(Arc::new(Some(player)));
    }

    #[expect(clippy::too_many_lines)]
    pub async fn progress_player_packets(&self, player: &Arc<Player>, server: &Arc<Server>) {
        let Some(mut network_reader) = self
            .network_reader
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
        else {
            return;
        };

        let keep_alive_time = server.advanced_config.networking.java.keep_alive_time;
        let mut keep_alive_interval =
            tokio::time::interval(std::time::Duration::from_secs(keep_alive_time.max(1)));
        let timeout_duration =
            std::time::Duration::from_secs(keep_alive_time.saturating_mul(2).max(1));

        // Skip the immediate first tick so we don't send a keep-alive the exact millisecond they join
        keep_alive_interval.tick().await;

        loop {
            tokio::select! {
                // KEEP-ALIVE TIMER
                _ = keep_alive_interval.tick() => {
                    // Check if the client has timed out on keep-alive responses or no packet activity
                    let has_timed_out = {
                        let pending = self
                            .pending_keep_alives
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        pending.iter().any(|(_, send_time)| send_time.elapsed() > timeout_duration)
                    } || (self.wait_for_keep_alive.load(Ordering::Relaxed) && self.last_keep_alive_time.load().elapsed() > timeout_duration)
                      || (self.last_packet_time.load().elapsed() > timeout_duration);

                    if has_timed_out {
                        self.kick(pumpkin_macros::translate_cross!(translation::java::DISCONNECT_TIMEOUT, translation::bedrock::DISCONNECT_TIMEOUT)).await;
                        break;
                    }

                    // Generate a unique ID (current timestamp in ms)
                    let keep_alive_id = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as i64;

                    self.keep_alive_id.store(keep_alive_id);
                    self.wait_for_keep_alive.store(true, Ordering::Relaxed);
                    self.last_keep_alive_time.store(Instant::now());
                    {
                        let mut pending = self
                            .pending_keep_alives
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        pending.push((keep_alive_id, Instant::now()));
                        if pending.len() > 16 {
                            pending.remove(0);
                        }
                    }
                    let packet = pumpkin_protocol::java::client::play::CKeepAlive::new(keep_alive_id);
                    self.enqueue_client_packet(&packet).await;
                }

                // INCOMING PACKETS
                packet_opt = self.get_packet_with_reader(&mut network_reader) => {
                    let Some(packet) = packet_opt else {
                        break;
                    };
                    self.last_packet_time.store(Instant::now());

                    if !self.packet_limiter.check_packet() {
                        warn!(
                            "Client {} ({}) exceeded packet rate limit (rate: {}/s)",
                            self.id,
                            self.gameprofile.name,
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
                        break;
                    }

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
                            error!(
                                "Failed to handle play packet id {} (payload {} bytes): {}",
                                packet.id,
                                packet.payload.len(),
                                e
                            );
                        }
                    }

                    // ServerGamePacketListenerImpl acknowledges the sequence at the end of the
                    // packet that carried it. Until we do, the client keeps predicting the block
                    // it interacted with and drops our updates for that position
                    let seq = self.packet_sequence.swap(-1, Ordering::Relaxed);
                    if seq != -1 {
                        self.send_packet(&CAcknowledgeBlockChange::new(seq.into()))
                            .await;
                    }
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

    pub async fn send_chunks(&self, chunks: &[SyncChunk]) {
        let player = self.player.load_full();
        let Some(player) = player.as_ref() else {
            return;
        };
        let Some(server) = player.world().server.upgrade() else {
            return;
        };

        if self.version.load() >= JavaMinecraftVersion::V_1_20_2 {
            self.send_packet(&CChunkBatchStart).await;
        }
        for chunk in chunks {
            let mut event = ChunkSend::new(player.world(), chunk.clone());
            server.plugin_manager.fire(&server, &mut event).await;
            if event.cancelled {
                continue;
            }

            let mut buf = Vec::new();
            let version = self.version.load();
            if let Err(err) = buf.write_var_int(&VarInt(CChunkData::to_id(version))) {
                error!("Failed to write chunk data id: {err:?}");
                continue;
            }
            if let Err(err) = CChunkData(chunk).write_packet_data(&mut buf, &version) {
                error!("Failed to write chunk data: {err:?}");
                continue;
            }
            self.send_packet_now_data(buf.into()).await;
        }
        if self.version.load() >= JavaMinecraftVersion::V_1_20_2 {
            self.send_packet(&CChunkBatchEnd::new(chunks.len() as u16))
                .await;
        }
    }

    pub async fn enqueue_packet(&self, packet_data: Bytes) {
        self.enqueue_packet_data(packet_data).await;
    }

    pub async fn enqueue_packet_data(&self, packet_data: Bytes) {
        if let Err(err) = self
            .outgoing_packet_queue_send
            .send(OutgoingPacket::normal(packet_data))
            .await
        {
            // This is expected to fail if we are closed
            if !self.close_token.is_cancelled() {
                warn!(
                    "Failed to add packet to the outgoing packet queue for client {}: {}",
                    self.id, err
                );
                // We now need to close the connection to the client since the stream is in an
                // unknown state
                self.close();
            }
        }
    }

    pub fn try_enqueue_packet(&self, packet_data: Bytes) {
        self.try_enqueue_packet_data(packet_data);
    }

    pub fn try_enqueue_packet_data(&self, packet_data: Bytes) {
        if let Err(err) = self
            .outgoing_packet_queue_send
            .try_send(OutgoingPacket::normal(packet_data))
        {
            match err {
                tokio::sync::mpsc::error::TrySendError::Full(_) => {
                    debug!(
                        "Failed to add packet to the outgoing packet queue for client {}: channel full",
                        self.id
                    );
                }
                tokio::sync::mpsc::error::TrySendError::Closed(_) => {
                    if !self.close_token.is_cancelled() {
                        warn!(
                            "Failed to add packet to the outgoing packet queue for client {}: channel closed",
                            self.id
                        );
                        self.close();
                    }
                }
            }
        }
    }

    pub async fn await_close_interrupt(&self) {
        self.close_token.cancelled().await;
    }

    pub async fn get_packet_with_reader(
        &self,
        network_reader: &mut TCPNetworkDecoder<BufReader<OwnedReadHalf>>,
    ) -> Option<RawPacket> {
        tokio::select! {
            () = self.await_close_interrupt() => {
                debug!("Canceling player packet processing");
                None
            },
            packet_result = network_reader.get_raw_packet() => {
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
        }
    }

    pub async fn kick(&self, reason: TextComponent) {
        self.kick_explicit(&reason, true).await;
    }

    pub async fn kick_explicit(&self, reason: &TextComponent, send_packet: bool) {
        if send_packet {
            match self.connection_state.load() {
                ConnectionState::Login => {
                    // TextComponent implements Serialize and writes in bytes instead of String, that's the reason we only use content
                    self.send_packet(&CLoginDisconnect::new(
                        serde_json::to_string(&reason.0).unwrap_or_else(|_| String::new()),
                    ))
                    .await;
                }
                ConnectionState::Config => {
                    self.send_packet(&CConfigDisconnect::new(&reason.clone().get_text()))
                        .await;
                }
                ConnectionState::Play => self.send_packet(&CPlayDisconnect::new(reason)).await,
                _ => {}
            }
        }
        debug!("Closing connection for {}", self.id);
        self.close();
    }

    pub async fn send_packet_now(&self, packet: Bytes) {
        self.send_packet_now_data(packet).await;
    }

    pub async fn send_packet_now_data(&self, packet: Bytes) {
        let (completion_tx, completion_rx) = oneshot::channel();

        if let Err(err) = self
            .outgoing_packet_priority_send
            .send(OutgoingPacket::high_priority(packet, completion_tx))
            .await
        {
            // It is expected that the packet will fail if we are closed
            if !self.close_token.is_cancelled() {
                warn!(
                    "Failed to add high-priority packet to the outgoing packet queue for client {}: {}",
                    self.id, err
                );
                // We now need to close the connection to the client since the stream is in an
                // unknown state
                self.close();
            }
            return;
        }

        if completion_rx.await.is_err() && !self.close_token.is_cancelled() {
            // The outgoing packet task dropped before confirming the write.
            self.close();
        }
    }

    pub fn write_packet_for_version<P: ClientPacket>(
        packet: &P,
        version: JavaMinecraftVersion,
        write: impl Write,
    ) -> Result<(), WritingError> {
        pumpkin_protocol::java::packet_encoder::write_packet(packet, &version, write)
    }

    pub fn serialize_packet_for_version<P: ClientPacket>(
        packet: &P,
        version: JavaMinecraftVersion,
    ) -> Result<Bytes, WritingError> {
        pumpkin_protocol::java::packet_encoder::serialize_packet(packet, &version)
    }

    pub fn serialize_packet<P: ClientPacket>(&self, packet: &P) -> Result<Bytes, WritingError> {
        Self::serialize_packet_for_version(packet, self.version.load())
    }

    pub async fn send_packet<P: ClientPacket>(&self, packet: &P) {
        if let Ok(data) = self.serialize_packet(packet) {
            self.send_packet_now(data).await;
        }
    }

    pub async fn enqueue_client_packet<P: ClientPacket>(&self, packet: &P) {
        if let Ok(data) = self.serialize_packet(packet) {
            self.enqueue_packet(data).await;
        }
    }

    pub fn write_packet<P: ClientPacket>(
        &self,
        packet: &P,
        write: impl Write,
    ) -> Result<(), WritingError> {
        Self::write_packet_for_version(packet, self.version.load(), write)
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
    pub fn start_outgoing_packet_task(&mut self) {
        const MAX_BATCH_SIZE: usize = 64;

        let Some(mut packet_receiver) = self.outgoing_packet_queue_recv.take() else {
            return;
        };
        let Some(mut priority_packet_receiver) = self.outgoing_packet_priority_recv.take() else {
            return;
        };
        let close_token = self.close_token.clone();
        let Some(mut writer) = self
            .network_writer
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
        else {
            return;
        };
        let id = self.id;
        self.spawn_task(async move {
            while !close_token.is_cancelled() {
                let recv_result = tokio::select! {
                    biased;
                    () = close_token.cancelled() => None,
                    res = priority_packet_receiver.recv() => res,
                    res = packet_receiver.recv() => res,
                };

                let Some(packet_data) = recv_result else {
                    break;
                };

                let mut packet_batch = Vec::with_capacity(MAX_BATCH_SIZE);
                packet_batch.push(packet_data);

                while packet_batch.len() < MAX_BATCH_SIZE {
                    match priority_packet_receiver.try_recv() {
                        Ok(packet_data) => {
                            packet_batch.push(packet_data);
                            continue;
                        }
                        Err(TryRecvError::Disconnected | TryRecvError::Empty) => {}
                    }

                    match packet_receiver.try_recv() {
                        Ok(packet_data) => packet_batch.push(packet_data),
                        Err(TryRecvError::Disconnected | TryRecvError::Empty) => break,
                    }
                }

                let send_failed = {
                    let mut failed = false;
                    for packet in &packet_batch {
                        if let Err(err) = writer.write_packet(packet.data.clone()).await {
                            failed = true;
                            // It is expected that the packet will fail if we are closed
                            if !close_token.is_cancelled() {
                                warn!("Failed to send packet to client {id}: {err}");
                            }
                            break;
                        }
                    }

                    if !failed && let Err(err) = writer.flush().await {
                        failed = true;
                        if !close_token.is_cancelled() {
                            warn!("Failed to flush packet batch for client {id}: {err}");
                        }
                    }
                    failed
                };

                if send_failed {
                    // We now need to close the connection to the client since the stream is in an unknown state.
                    close_token.cancel();
                    break;
                }

                for packet in packet_batch {
                    if let Some(completion) = packet.completion {
                        let _ = completion.send(());
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

    #[expect(clippy::too_many_lines)]
    pub async fn handle_play_packet(
        &self,
        player: &Arc<Player>,
        server: &Arc<Server>,
        packet: &RawPacket,
    ) -> Result<(), Box<dyn PumpkinError>> {
        let version = self.version.load();

        let mut event = crate::plugin::server::packet::PacketReceivedEvent::new(
            player.clone(),
            packet.id,
            packet.payload.clone(),
        );
        server.plugin_manager.fire(server, &mut event).await;
        if event.cancelled {
            return Ok(());
        }

        let mut payload = &event.payload[..];

        match event.packet_id {
            id if id == SConfirmTeleport::to_id(version) => {
                self.handle_confirm_teleport(
                    player,
                    SConfirmTeleport::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SChangeGameMode::to_id(version) => {
                self.handle_change_game_mode(
                    player,
                    SChangeGameMode::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SChatCommand::to_id(version) => {
                self.handle_chat_command(
                    player,
                    server,
                    &(SChatCommand::read(&mut payload, &version)?),
                )
                .await;
            }
            id if id == SChatMessage::to_id(version) => {
                self.handle_chat_message(
                    server,
                    player,
                    SChatMessage::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SClientInformationPlay::to_id(version) => {
                self.handle_client_information(
                    server,
                    player,
                    SClientInformationPlay::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SClientCommand::to_id(version) => {
                self.handle_client_status(player, SClientCommand::read(&mut payload, &version)?)
                    .await;
            }
            id if id == SPlayerInput::to_id(version) => {
                self.handle_player_input(
                    player,
                    SPlayerInput::read(&mut payload, &version)?,
                    server,
                )
                .await;
            }
            id if id == SMoveVehicle::to_id(version) => {
                self.handle_move_vehicle(player, SMoveVehicle::read(&mut payload, &version)?)
                    .await;
            }
            id if id == SPaddleBoat::to_id(version) => {
                self.handle_paddle_boat(player, SPaddleBoat::read(&mut payload, &version)?)
                    .await;
            }
            id if id == SInteract::to_id(version) => {
                self.handle_interact(player, SInteract::read(&mut payload, &version)?, server)
                    .await;
            }
            id if id == SBundleItemSelected::to_id(version) => {
                self.handle_bundle_item_selected(
                    player,
                    SBundleItemSelected::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SAttack::to_id(version) => {
                self.handle_attack(player, SAttack::read(&mut payload, &version)?, server)
                    .await;
            }
            id if id == STeleportToEntity::to_id(version) => {
                self.handle_teleport_to_entity(
                    player,
                    STeleportToEntity::read(&mut payload, &version)?,
                    server,
                )
                .await;
            }
            id if id == pumpkin_protocol::java::server::play::SKeepAlive::to_id(version) => {
                self.handle_keep_alive(
                    player,
                    &pumpkin_protocol::java::server::play::SKeepAlive::read(
                        &mut payload,
                        &version,
                    )?,
                );
            }
            id if id == SClientTickEnd::to_id(version) => {
                // TODO
            }
            id if id == STestInstanceBlockAction::to_id(version) => {
                self.handle_test_instance_block_action(
                    player,
                    &STestInstanceBlockAction::read(&mut payload, &version)?,
                );
            }
            id if id == SSetTestBlock::to_id(version) => {
                self.handle_set_test_block(player, &SSetTestBlock::read(&mut payload, &version)?);
            }
            id if id == SDebugSubscriptionRequest::to_id(version) => {
                self.handle_debug_subscription_request(
                    player,
                    &SDebugSubscriptionRequest::read(&mut payload, &version)?,
                );
            }
            id if id == SDebugSampleSubscription::to_id(version) => {
                self.handle_debug_sample_subscription(
                    player,
                    &SDebugSampleSubscription::read(&mut payload, &version)?,
                );
            }
            id if id == SPlayerPosition::to_id(version) => {
                self.handle_position(
                    player,
                    server,
                    SPlayerPosition::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SPlayerPositionRotation::to_id(version) => {
                self.handle_position_rotation(
                    player,
                    server,
                    SPlayerPositionRotation::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SPlayerRotation::to_id(version) => {
                self.handle_rotation(player, SPlayerRotation::read(&mut payload, &version)?)
                    .await;
            }
            id if id == SSetPlayerGround::to_id(version) => {
                self.handle_player_ground(player, &SSetPlayerGround::read(&mut payload, &version)?);
            }
            id if id == SPickItemFromBlock::to_id(version) => {
                self.handle_pick_item_from_block(
                    player,
                    SPickItemFromBlock::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id
                == pumpkin_protocol::java::server::play::SPickItemFromEntity::to_id(version) =>
            {
                self.handle_pick_item_from_entity(
                    player,
                    pumpkin_protocol::java::server::play::SPickItemFromEntity::read(
                        &mut payload,
                        &version,
                    )?,
                )
                .await;
            }
            id if id == SPlayerAbilities::to_id(version) => {
                self.handle_player_abilities(
                    player,
                    SPlayerAbilities::read(&mut payload, &version)?,
                    server,
                )
                .await;
            }
            id if id == SPlayerAction::to_id(version) => {
                self.handle_player_action(
                    player,
                    SPlayerAction::read(&mut payload, &version)?,
                    server,
                )
                .await;
            }
            id if id == SSetCommandBlock::to_id(version) => {
                self.handle_set_command_block(
                    player,
                    SSetCommandBlock::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SSetJigsawBlock::to_id(version) => {
                self.handle_set_jigsaw_block(
                    player,
                    SSetJigsawBlock::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SJigsawGenerate::to_id(version) => {
                self.handle_jigsaw_generate(player, SJigsawGenerate::read(&mut payload, &version)?)
                    .await;
            }
            id if id == SPlayerCommand::to_id(version) => {
                self.handle_player_command(
                    player,
                    SPlayerCommand::read(&mut payload, &version)?,
                    server,
                )
                .await;
            }
            id if id == SPlayerLoaded::to_id(version) => {
                Self::handle_player_loaded(player);
            }
            id if id == SPlayPingRequest::to_id(version) => {
                self.handle_play_ping_request(SPlayPingRequest::read(&mut payload, &version)?)
                    .await;
            }
            id if id == SClickSlot::to_id(version) => {
                player
                    .on_slot_click(SClickSlot::read(&mut payload, &version)?, server)
                    .await;
            }
            id if id == SContainerButtonClick::to_id(version) => {
                player
                    .on_container_button_click(SContainerButtonClick::read(&mut payload, &version)?)
                    .await;
            }
            id if id == SSetHeldItem::to_id(version) => {
                self.handle_set_held_item(
                    server,
                    player,
                    SSetHeldItem::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SSetCreativeSlot::to_id(version) => {
                self.handle_set_creative_slot(
                    player,
                    SSetCreativeSlot::read(&mut payload, &version)?,
                )
                .await?;
            }
            id if id == SSwingArm::to_id(version) => {
                self.handle_swing_arm(server, player, SSwingArm::read(&mut payload, &version)?)
                    .await;
            }
            id if id == SUpdateSign::to_id(version) => {
                self.handle_sign_update(player, SUpdateSign::read(&mut payload, &version)?)
                    .await;
            }
            id if id == SEditBook::to_id(version) => {
                self.handle_edit_book(player, SEditBook::read(&mut payload, &version)?)
                    .await;
            }
            id if id == SUseItemOn::to_id(version) => {
                self.handle_use_item_on(player, SUseItemOn::read(&mut payload, &version)?, server)
                    .await?;
            }
            id if id == SUseItem::to_id(version) => {
                self.handle_use_item(player, &SUseItem::read(&mut payload, &version)?, server)
                    .await;
            }
            id if id == SCommandSuggestion::to_id(version) => {
                self.handle_command_suggestion(
                    player,
                    SCommandSuggestion::read(&mut payload, &version)?,
                    server,
                )
                .await;
            }
            id if id == SPCookieResponse::to_id(version) => {
                self.handle_cookie_response(&SPCookieResponse::read(&mut payload, &version)?);
            }
            id if id == SCloseContainer::to_id(version) => {
                self.handle_close_container(
                    player,
                    server,
                    SCloseContainer::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SChunkBatch::to_id(version) => {
                self.handle_chunk_batch(player, SChunkBatch::read(&mut payload, &version)?)
                    .await;
            }
            id if id == SPlayerSession::to_id(version) => {
                self.handle_chat_session_update(
                    player,
                    server,
                    SPlayerSession::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SCustomPayload::to_id(version) => {
                let payload = SCustomPayload::read(&mut payload, &version)?;
                let mut event = PlayerCustomPayloadEvent::new(
                    player.clone(),
                    payload.channel.to_string(),
                    Bytes::copy_from_slice(payload.data),
                );
                server.plugin_manager.fire(server, &mut event).await;
            }
            id if id == SRecipeBookChangeSettings::to_id(version) => {
                self.handle_recipe_book_change_settings(
                    server,
                    player,
                    SRecipeBookChangeSettings::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SRecipeBookSeenRecipe::to_id(version) => {
                self.handle_recipe_book_seen_recipe(
                    server,
                    player,
                    SRecipeBookSeenRecipe::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SRenameItem::to_id(version) => {
                player
                    .on_rename_item(SRenameItem::read(&mut payload, &version)?)
                    .await;
            }
            id if id == SPlaceRecipe::to_id(version) => {
                let packet = SPlaceRecipe::read(&mut payload, &version)?;
                self.handle_place_recipe(server, player, packet).await;
            }
            id if id
                == pumpkin_protocol::java::server::play::SCustomClickAction::to_id(version) =>
            {
                let packet = pumpkin_protocol::java::server::play::SCustomClickAction::read(
                    &mut payload,
                    &version,
                )?;
                let mut event = crate::plugin::api::events::player::custom_click_action::CustomClickActionEvent::new(
                    player.clone(),
                    packet.action_id.to_string(),
                    packet.payload.map(Bytes::copy_from_slice),
                );
                server.plugin_manager.fire(server, &mut event).await;
            }
            id if id == SSelectTrade::to_id(version) => {
                self.handle_select_trade(player, SSelectTrade::read(&mut payload, &version)?)
                    .await;
            }
            id if id == SSeenAdvancement::to_id(version) => {
                self.handle_seen_advancement(
                    player,
                    SSeenAdvancement::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SPlayResourcePack::to_id(version) => {
                self.handle_play_resource_pack_response(
                    server,
                    player,
                    SPlayResourcePack::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SPlayPong::to_id(version) => {
                self.handle_play_pong(player, &SPlayPong::read(&mut payload, &version)?);
            }
            id if id == SLockDifficulty::to_id(version) => {
                self.handle_lock_difficulty(
                    server,
                    player,
                    &SLockDifficulty::read(&mut payload, &version)?,
                );
            }
            id if id == SContainerSlotStateChanged::to_id(version) => {
                self.handle_container_slot_state_changed(
                    player,
                    &SContainerSlotStateChanged::read(&mut payload, &version)?,
                );
            }
            id if id == SSpectateEntity::to_id(version) => {
                self.handle_spectate_entity(
                    player,
                    server,
                    SSpectateEntity::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SSetCommandMinecart::to_id(version) => {
                self.handle_set_command_minecart(
                    player,
                    &SSetCommandMinecart::read(&mut payload, &version)?,
                );
            }
            id if id == SSetStructureBlock::to_id(version) => {
                self.handle_set_structure_block(
                    player,
                    &SSetStructureBlock::read(&mut payload, &version)?,
                );
            }
            id if id == SSetGameRule::to_id(version) => {
                self.handle_set_game_rule(player, &SSetGameRule::read(&mut payload, &version)?);
            }
            id if id == SBlockEntityTagQuery::to_id(version) => {
                self.handle_block_entity_tag_query(
                    player,
                    SBlockEntityTagQuery::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SEntityTagQuery::to_id(version) => {
                self.handle_entity_tag_query(
                    player,
                    SEntityTagQuery::read(&mut payload, &version)?,
                )
                .await;
            }
            id if id == SConfigurationAcknowledged::to_id(version) => {
                self.handle_configuration_acknowledged(player);
            }
            _ => {
                warn!("Failed to handle player packet id {}", event.packet_id);
            }
        }
        Ok(())
    }
}
