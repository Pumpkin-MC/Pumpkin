pub mod nethernet;
pub mod play;
pub(crate) mod recipe;
use crossbeam::atomic::AtomicCell;
use std::{
    collections::{HashMap, VecDeque},
    io::{Cursor, Error, Write},
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering},
    },
    time::Instant,
};

use tracing::{debug, error, warn};

use bytes::Bytes;
use pumpkin_config::networking::compression::CompressionInfo;
use pumpkin_protocol::{
    BClientPacket, PacketDecodeError, RawPacket,
    bedrock::{
        BEDROCK_GAME_PACKET, SubClient,
        client::{
            client_cache_miss_response::{CClientCacheMissResponse, MissingBlobData},
            disconnect::CDisconnect,
        },
        network_stack_latency::NetworkStackLatency,
        packet_decoder::BedrockBatchDecoder,
        packet_encoder::BedrockBatchEncoder,
        server::{
            actor_event::SActorEvent, animate::SAnimate, block_actor_data::SBlockActorData,
            block_pick_request::SBlockPickRequest,
            client_cache_blob_status::SClientCacheBlobStatus,
            client_cache_status::SClientCacheStatus,
            client_movement_prediction_sync::SClientMovementPredictionSync,
            command_request::SCommandRequest, container_close::SContainerClose,
            cursor_item_drag::SCursorItemDrag, emote::SEmote, emote_list::SEmoteList,
            interact::SInteract, inventory_transaction::SInventoryTransaction,
            item_stack_request::SItemStackRequest, loading_screen::SLoadingScreen, login::SLogin,
            matchmaking_cancel::SMatchmakingCancel, mob_equipment::SMobEquipment,
            modal_form_response::SModalFormResponse,
            packet_violation_warning::SPacketViolationWarning, player_action::SPlayerAction,
            player_auth_input::SPlayerAuthInput, player_hotbar::SPlayerHotbar,
            register_audio_content::SRegisterAudioContent, request_ability::SRequestAbility,
            request_chunk_radius::SRequestChunkRadius,
            request_network_settings::SRequestNetworkSettings,
            resource_pack_chunk_request::SResourcePackChunkRequest,
            resource_pack_client_response::SResourcePackClientResponse, respawn::SRespawn,
            set_local_player_as_initialized::SSetLocalPlayerAsInitialized,
            set_player_inventory_options::SSetPlayerInventoryOptions,
            stonecutter_set_recipe::SStonecutterSetRecipe, text::SText,
        },
    },
    packet::Packet,
    serial::{PacketRead, PacketReadSlice, PacketWrite},
};
use tokio::{
    sync::mpsc::{Receiver, Sender, UnboundedReceiver, UnboundedSender, error::TryRecvError},
    sync::{Mutex, RwLock, oneshot},
    task::JoinHandle,
};

use tokio_util::{sync::CancellationToken, task::TaskTracker};

pub mod blob_cache;
pub mod level_chunk;
pub mod login;
pub mod versions;
use self::level_chunk::CLevelChunk;
use self::nethernet::NetherNetSession;
use crate::{
    entity::player::Player,
    net::{
        DisconnectReason, MAX_PENDING_BYTES, PacketHandlerResult, PacketRateLimiter,
        chunk_sender::{ChunkSendOutcome, DeliveryFailure},
        decrement_pending_bytes,
    },
    plugin::{
        api::events::world::chunk_send::ChunkSend,
        server::packet::{
            BedrockConnectionPacketReceivedEvent, BedrockConnectionPacketSentEvent, PacketSentEvent,
        },
    },
    server::Server,
};
use arc_swap::ArcSwap;
use pumpkin_protocol::{bedrock::server::login::ClientData, codec::var_uint::VarUInt};
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::version::BedrockMinecraftVersion;
use pumpkin_world::{CURRENT_BEDROCK_MC_PROTOCOL, level::SyncChunk};
pub use versions::BedrockVersions;

pub struct OutgoingPacket {
    pub data: Bytes,
    pub completion: Option<oneshot::Sender<()>>,
}

impl OutgoingPacket {
    pub const fn normal(data: Bytes) -> Self {
        Self {
            data,
            completion: None,
        }
    }

    pub const fn priority(data: Bytes, completion: oneshot::Sender<()>) -> Self {
        Self {
            data,
            completion: Some(completion),
        }
    }
}

/// Active resource pack staged for chunk streaming to this client.
#[derive(Clone)]
pub struct BedrockStreamingPack {
    pub pack_id: String,
    pub chunk_size: u32,
    pub chunk_count: u32,
    pub total_size: u64,
    pub data: Arc<Vec<u8>>,
}

type TranslatedPacket = (Option<Bytes>, Vec<Bytes>);

/// Frames a raw Bedrock packet as `VarUInt(packet_id) + payload` for the outgoing queue.
#[must_use]
pub fn frame_raw_packet(packet_id: i32, payload: &[u8]) -> Option<Bytes> {
    let id = u16::try_from(packet_id).ok()?;
    let id_varint = VarUInt(u32::from(id));
    let mut buf = Vec::with_capacity(id_varint.written_size() + payload.len());
    id_varint.write(&mut buf).ok()?;
    buf.extend_from_slice(payload);
    Some(Bytes::from(buf))
}

/// Splits an internal `VarUInt(packet_id) + payload` buffer back into `(packet_id, payload)`.
#[must_use]
pub fn split_raw_packet(packet_data: &Bytes) -> Option<(u16, Bytes)> {
    let mut reader = &packet_data[..];
    let packet_id = VarUInt::read(&mut reader).ok()?.0;
    let packet_id = u16::try_from(packet_id).ok()?;
    let payload = packet_data.slice(packet_data.len() - reader.len()..);
    Some((packet_id, payload))
}

pub struct BedrockClient {
    pub id: u64,
    session: Arc<NetherNetSession>,
    /// The client's IP address.
    pub address: SocketAddr,
    pub player: ArcSwap<Option<Arc<Player>>>,
    pub protocol_version: AtomicU32,
    pub version: AtomicCell<BedrockMinecraftVersion>,
    pub client_data: ArcSwap<Option<Arc<ClientData>>>,
    server: std::sync::Weak<Server>,
    /// All Bedrock clients
    /// This list is used to remove the client if the connection gets closed
    pub be_clients: Arc<Mutex<HashMap<SocketAddr, Arc<Self>>>>,

    tasks: TaskTracker,
    rt_handle: tokio::runtime::Handle,
    outgoing_packet_queue_send: UnboundedSender<OutgoingPacket>,
    /// FIFO queue of serialized packets for the network
    outgoing_packet_queue_recv: Mutex<Option<UnboundedReceiver<OutgoingPacket>>>,

    /// Tracks total buffered payload bytes in the outgoing queues.
    pub pending_bytes: Arc<AtomicUsize>,

    /// The packet encoder for outgoing packets.
    network_writer: Arc<RwLock<BedrockBatchEncoder>>,
    /// The packet decoder for incoming packets.
    network_reader: Mutex<BedrockBatchDecoder>,

    /// The next form ID to use for custom forms.
    pub next_form_id: AtomicU32,
    pub inventory_opened: AtomicBool,
    /// Last processed `PlayerAuthInput` tick. Motion for the own player must carry it.
    pub input_tick: AtomicU64,
    /// Client predicted velocity from the last `PlayerAuthInput`. Pushes build on it.
    pub client_delta: AtomicCell<pumpkin_util::math::vector3::Vector3<f64>>,
    /// World age of the last server motion sent to the own player.
    pub own_motion_age: AtomicCell<i64>,
    /// `PlayerAuthInput` tick tagged on that last own-player `SetActorMotion`.
    pub own_motion_tick: AtomicU64,
    /// Push velocity the client carries from the server's motions.
    pub push_residual: std::sync::Mutex<crate::entity::player::client_push::PushResidual>,
    /// Ids and send times of unanswered `NetworkStackLatency` probes, oldest first.
    pending_latencies: std::sync::Mutex<VecDeque<(u64, Instant)>>,
    next_latency_id: AtomicU64,
    last_latency_send: AtomicCell<Instant>,
    /// Inputs since the last teleport without reaching it. Resent past a threshold.
    pub teleport_unconfirmed_inputs: AtomicU32,
    /// Network position of the last own teleport `MovePlayer`, which the client must reach.
    pub teleport_sent_pos: AtomicCell<pumpkin_util::math::vector3::Vector3<f32>>,
    /// Separate from normal vitals caching so the first rejected use always gets corrected.
    last_food_rejection_tick: AtomicCell<Option<i32>>,
    pub client_cache_supported: AtomicBool,
    pub blob_cache: std::sync::Mutex<blob_cache::BlobCache>,
    pub streaming_packs: std::sync::Mutex<HashMap<String, BedrockStreamingPack>>,
    /// An notifier that is triggered when this client is closed.
    close_token: CancellationToken,
    last_seen: Arc<AtomicCell<std::time::Instant>>,
    incoming_game_packet_send: Sender<RawPacket>,
    incoming_game_packet_recv: Mutex<Option<Receiver<RawPacket>>>,
    /// Packet rate limiter for incoming client packets.
    pub packet_limiter: PacketRateLimiter,
}

impl BedrockClient {
    #[must_use]
    pub fn new(
        id: u64,
        session: Arc<NetherNetSession>,
        address: SocketAddr,
        be_clients: Arc<Mutex<HashMap<SocketAddr, Arc<Self>>>>,
        packet_limiter: PacketRateLimiter,
        server: std::sync::Weak<Server>,
    ) -> Self {
        let (send, recv) = tokio::sync::mpsc::unbounded_channel();
        let (incoming_send, incoming_recv) = tokio::sync::mpsc::channel(4096);
        let rt_handle = tokio::runtime::Handle::current();
        Self {
            id,
            session,
            player: ArcSwap::new(Arc::new(None)),
            address,
            protocol_version: AtomicU32::new(0),
            version: AtomicCell::new(BedrockMinecraftVersion::Unknown),
            client_data: ArcSwap::new(Arc::new(None)),
            server,
            be_clients,
            network_writer: Arc::new(RwLock::new(BedrockBatchEncoder::new())),
            network_reader: Mutex::new(BedrockBatchDecoder::new()),
            tasks: TaskTracker::new(),
            rt_handle,
            outgoing_packet_queue_send: send,
            outgoing_packet_queue_recv: Mutex::new(Some(recv)),
            pending_bytes: Arc::new(AtomicUsize::new(0)),
            next_form_id: AtomicU32::new(0),
            inventory_opened: AtomicBool::new(false),
            input_tick: AtomicU64::new(0),
            client_delta: AtomicCell::new(pumpkin_util::math::vector3::Vector3::default()),
            own_motion_age: AtomicCell::new(i64::MIN),
            own_motion_tick: AtomicU64::new(0),
            push_residual: std::sync::Mutex::default(),
            pending_latencies: std::sync::Mutex::new(VecDeque::new()),
            next_latency_id: AtomicU64::new(1),
            last_latency_send: AtomicCell::new(Instant::now()),
            teleport_unconfirmed_inputs: AtomicU32::new(0),
            teleport_sent_pos: AtomicCell::new(pumpkin_util::math::vector3::Vector3::default()),
            last_food_rejection_tick: AtomicCell::new(None),
            client_cache_supported: AtomicBool::new(false),
            blob_cache: std::sync::Mutex::new(blob_cache::BlobCache::new()),
            streaming_packs: std::sync::Mutex::new(HashMap::new()),
            close_token: CancellationToken::new(),
            last_seen: Arc::new(AtomicCell::new(std::time::Instant::now())),
            incoming_game_packet_send: incoming_send,
            incoming_game_packet_recv: Mutex::new(Some(incoming_recv)),
            packet_limiter,
        }
    }

    pub async fn get_packet(&self) -> Option<RawPacket> {
        let mut guard = self.incoming_game_packet_recv.lock().await;
        let recv = guard.as_mut()?;
        tokio::select! {
            () = self.await_close_interrupt() => None,
            packet = recv.recv() => packet,
        }
    }

    pub fn start_outgoing_packet_task(self: &Arc<Self>) {
        const MAX_BATCH_SIZE: usize = 64;

        let client = self.clone();
        self.spawn_task(async move {
            let Some(mut packet_receiver) = client.outgoing_packet_queue_recv.lock().await.take()
            else {
                return;
            };
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(100));

            loop {
                let recv_result = tokio::select! {
                    biased;
                    res = packet_receiver.recv() => res,
                    _ = interval.tick() => {
                        if !client.tick_connection().await {
                            break;
                        }
                        continue;
                    }
                    () = client.close_token.cancelled() => {
                        packet_receiver.try_recv().ok()
                    }
                };

                let Some(packet) = recv_result else {
                    break;
                };

                let mut packet_batch = Vec::with_capacity(MAX_BATCH_SIZE);
                packet_batch.push(packet);

                while packet_batch.len() < MAX_BATCH_SIZE {
                    match packet_receiver.try_recv() {
                        Ok(packet) => packet_batch.push(packet),
                        Err(TryRecvError::Disconnected | TryRecvError::Empty) => break,
                    }
                }

                let mut wire_buf = Vec::new();
                for packet in packet_batch {
                    let packet_len = packet.data.len();
                    let Some((packet_id, payload)) = split_raw_packet(&packet.data) else {
                        warn!(
                            "Dropping malformed outgoing Bedrock packet for {}",
                            client.address
                        );
                        decrement_pending_bytes(&client.pending_bytes, packet_len);
                        continue;
                    };
                    wire_buf.clear();
                    let encode_res = client.network_writer.read().await.write_game_packet(
                        packet_id,
                        SubClient::Main,
                        SubClient::Main,
                        &payload,
                        &mut wire_buf,
                    );
                    if let Err(error) = encode_res {
                        error!(
                            "Failed to encode Bedrock packet {packet_id} for {}: {error}",
                            client.address
                        );
                        decrement_pending_bytes(&client.pending_bytes, packet_len);
                        continue;
                    }
                    let Some(data) = wire_buf.strip_prefix(&[BEDROCK_GAME_PACKET]) else {
                        decrement_pending_bytes(&client.pending_bytes, packet_len);
                        continue;
                    };
                    if let Err(error) = client.session.send(Bytes::copy_from_slice(data)).await {
                        warn!(
                            "Failed to send NetherNet packet to {}: {error}",
                            client.address
                        );
                        decrement_pending_bytes(&client.pending_bytes, packet_len);
                        client.close().await;
                        return;
                    }

                    decrement_pending_bytes(&client.pending_bytes, packet_len);

                    if let Some(completion) = packet.completion {
                        let _ = completion.send(());
                    }
                }
            }
        });
    }

    async fn tick_connection(&self) -> bool {
        if self.last_seen.load().elapsed() > std::time::Duration::from_secs(10) {
            debug!("Bedrock client {} timed out", self.address);
            self.close().await;
            return false;
        }
        true
    }

    pub async fn process_nethernet_packet(self: &Arc<Self>, server: &Arc<Server>, packet: Bytes) {
        self.last_seen.store(std::time::Instant::now());
        let mut batch = Vec::with_capacity(packet.len() + 1);
        batch.push(BEDROCK_GAME_PACKET);
        batch.extend_from_slice(&packet);
        if let Err(error) = self.process_batch(server, batch).await {
            error!(
                "Failed to handle NetherNet payload for {}: {error}",
                self.address
            );
            self.kick(DisconnectReason::BadPacket, error.to_string())
                .await;
        }
    }

    pub fn nethernet_public_key(&self) -> Option<&pumpkin_auth::p384::PublicKey> {
        self.session.client_public_key()
    }

    pub async fn set_compression(&self, compression: CompressionInfo) {
        self.network_reader
            .lock()
            .await
            .set_compression(compression.threshold as usize);

        self.network_writer
            .write()
            .await
            .set_compression((compression.threshold as usize, compression.level));
    }

    pub async fn set_prefixed_compression(&self, prefixed: bool) {
        self.network_reader
            .lock()
            .await
            .set_prefixed_compression(prefixed);

        self.network_writer
            .write()
            .await
            .set_prefixed_compression(prefixed);
    }

    pub fn try_kick(&self, reason: DisconnectReason, message: String) {
        warn!("Closing connection for {}: {message}", self.address);
        let packet = CDisconnect::new(reason as i32, message);
        if let Some(data) = self
            .serialize_packet(&packet)
            .ok()
            .and_then(|data| self.translate_outgoing(data, false).0)
        {
            let packet_len = data.len();
            let _ = self.pending_bytes.fetch_add(packet_len, Ordering::AcqRel);
            let _ = self
                .outgoing_packet_queue_send
                .send(OutgoingPacket::normal(data));
        }
        if !self.close_token.is_cancelled() {
            self.close_token.cancel();
        }
    }

    pub async fn kick(&self, reason: DisconnectReason, message: String) {
        warn!("Closing connection for {}: {message}", self.address);
        self.send_packet(&CDisconnect::new(reason as i32, message))
            .await;
        let _ = self
            .session
            .flush_reliable(std::time::Duration::from_secs(1))
            .await;
        self.close().await;
    }

    pub async fn kick_explicit(
        &self,
        reason: DisconnectReason,
        message: String,
        skip_message: bool,
        filtered_message: String,
        send_packet: bool,
    ) {
        warn!("Closing connection for {}: {message}", self.address);
        if send_packet {
            self.send_packet(&CDisconnect {
                reason: pumpkin_protocol::codec::var_int::VarInt(reason as i32),
                skip_message,
                message,
                filtered_message,
            })
            .await;
            let _ = self
                .session
                .flush_reliable(std::time::Duration::from_secs(1))
                .await;
        }
        self.close().await;
    }

    /// Queues the chunks and reports which were sent and why the others were not.
    /// With an `epoch`, chunks are dropped if the player's chunk epoch moved on (world change)
    /// while they were being encoded.
    pub async fn send_chunks(&self, chunks: &[SyncChunk], epoch: Option<u32>) -> ChunkSendOutcome {
        let mut outcome = ChunkSendOutcome::default();
        let player = self.player.load_full();
        let Some(player) = player.as_ref() else {
            debug!(
                "send_chunks: player not set yet, deferring {} chunks",
                chunks.len()
            );
            return outcome;
        };
        let Some(server) = player.world().server.upgrade() else {
            return outcome;
        };

        let valid_chunks = ChunkSend::filter(&server, &player.world(), chunks).await;
        outcome.record_cancelled(chunks, &valid_chunks);
        if valid_chunks.is_empty() {
            return outcome;
        }
        let valid_positions: Vec<_> = valid_chunks
            .iter()
            .map(|chunk| Vector2::new(chunk.x, chunk.z))
            .collect();

        let bedrock_dimension =
            if player.world().dimension == pumpkin_data::dimension::Dimension::THE_NETHER {
                1
            } else if player.world().dimension == pumpkin_data::dimension::Dimension::THE_END {
                2
            } else {
                0
            };

        let cache_enabled = server.advanced_config.networking.bedrock.chunk_caching
            && self.client_cache_supported.load(Ordering::Relaxed);

        let world = player.world();
        let (tx, rx) = tokio::sync::oneshot::channel();
        server.compute_pool.spawn(move || {
            let mut encoded_payloads = Vec::with_capacity(valid_chunks.len());
            let mut new_blobs = Vec::new();
            for chunk in valid_chunks {
                let block_actors = world.bedrock_chunk_block_actors(&chunk);
                match CLevelChunk::encode_chunk(
                    &chunk,
                    bedrock_dimension,
                    cache_enabled,
                    &block_actors,
                ) {
                    Ok((payload, blobs)) => {
                        encoded_payloads.push((Vector2::new(chunk.x, chunk.z), Some(payload)));
                        new_blobs.extend(blobs);
                    }
                    Err(e) => {
                        error!("Failed to serialize Bedrock chunk: {:?}", e);
                        encoded_payloads.push((Vector2::new(chunk.x, chunk.z), None));
                    }
                }
            }
            let _ = tx.send((encoded_payloads, new_blobs));
        });

        let Ok((encoded_payloads, new_blobs)) = rx.await else {
            // Encoding task died -> counts as an encode failure for every chunk
            outcome.failed.extend(
                valid_positions
                    .into_iter()
                    .map(|pos| (pos, DeliveryFailure::Unencodable)),
            );
            return outcome;
        };

        let translated_chunks = self.translate_chunk_payloads(encoded_payloads).await;

        // Checked and queued under the sender lock: a world change resets the sender under it
        // before queueing `CChangeDimension`, so no old-world chunk can land behind that packet
        let _sender = epoch.map(|_| {
            player
                .chunk_sender
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        });
        if epoch.is_some_and(|epoch| player.chunk_send_epoch.load(Ordering::Relaxed) != epoch) {
            outcome.failed.extend(
                valid_positions
                    .into_iter()
                    .map(|pos| (pos, DeliveryFailure::Deferred)),
            );
            return outcome;
        }

        if !new_blobs.is_empty() {
            let mut cache = self
                .blob_cache
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for (hash, blob) in new_blobs {
                cache.insert(hash, blob);
            }
        }

        self.queue_translated_chunks(translated_chunks, &mut outcome);
        outcome
    }

    async fn translate_chunk_payloads(
        &self,
        encoded_payloads: Vec<(Vector2<i32>, Option<Vec<u8>>)>,
    ) -> Vec<(Vector2<i32>, Option<TranslatedPacket>)> {
        let mut translated_chunks = Vec::with_capacity(encoded_payloads.len());
        for (pos, payload) in encoded_payloads {
            let Some(framed) = payload.and_then(|p| frame_raw_packet(CLevelChunk::PACKET_ID, &p))
            else {
                translated_chunks.push((pos, None));
                continue;
            };
            let translated = self.translate_outgoing_async(framed, true).await;
            translated_chunks.push((pos, Some(translated)));
        }
        translated_chunks
    }

    fn queue_translated_chunks(
        &self,
        translated_chunks: Vec<(Vector2<i32>, Option<TranslatedPacket>)>,
        outcome: &mut ChunkSendOutcome,
    ) {
        for (pos, translated) in translated_chunks {
            let queued = match translated {
                Some((Some(packet), extras)) => Some(self.queue_translated_packet(packet, extras)),
                // Cancelled by a PacketSentEvent handler: queue any replacement packets and mark
                // sent so the chunk sender does not endlessly retry an intentionally replaced chunk.
                Some((None, extras)) => Some(self.queue_extra_packets(extras)),
                None => None,
            };
            match queued {
                Some(true) => outcome.sent.push(pos),
                Some(false) => outcome.failed.push((pos, DeliveryFailure::Deferred)),
                None => outcome.failed.push((pos, DeliveryFailure::Unencodable)),
            }
        }
    }

    pub fn set_player(&self, player: Arc<Player>) {
        self.player.store(Arc::new(Some(player)));
    }

    pub fn clear_player(&self) {
        self.player.store(Arc::new(None));
    }

    fn is_current_protocol(&self) -> bool {
        let protocol = self.protocol_version.load(Ordering::Relaxed);
        protocol == 0 || protocol == CURRENT_BEDROCK_MC_PROTOCOL
    }

    fn prepare_play_sent_event(
        &self,
        packet_data: &Bytes,
        observe: bool,
    ) -> Option<(Arc<Server>, PacketSentEvent, bool)> {
        let current_version = self.is_current_protocol();
        if current_version && !observe {
            return None;
        }
        let player_guard = self.player.load();
        let player = (**player_guard).as_ref()?;
        let server = player
            .world()
            .server
            .upgrade()
            .or_else(|| self.server.upgrade())?;
        if !server.plugin_manager.has_handlers::<PacketSentEvent>() {
            return None;
        }
        let (packet_id, payload) = split_raw_packet(packet_data)?;
        let event = PacketSentEvent::new_raw(player.clone(), i32::from(packet_id), payload);
        Some((server, event, current_version))
    }

    fn finish_play_sent_event(
        packet_data: Bytes,
        event: &PacketSentEvent,
        current_version: bool,
    ) -> TranslatedPacket {
        let extras = event
            .extra_packets
            .iter()
            .filter_map(|(id, payload)| frame_raw_packet(*id, payload))
            .collect();
        if event.cancelled {
            return (None, extras);
        }
        if current_version {
            return (Some(packet_data), extras);
        }
        (frame_raw_packet(event.packet_id, &event.payload), extras)
    }

    fn prepare_connection_sent_event(
        &self,
        packet_data: &Bytes,
    ) -> Option<(Arc<Server>, BedrockConnectionPacketSentEvent)> {
        if self.is_current_protocol() || self.player.load().is_some() {
            return None;
        }
        let server = self.server.upgrade()?;
        if !server
            .plugin_manager
            .has_handlers::<BedrockConnectionPacketSentEvent>()
        {
            return None;
        }
        let (packet_id, payload) = split_raw_packet(packet_data)?;
        let event = BedrockConnectionPacketSentEvent::new(
            self.id,
            self.protocol_version.load(Ordering::Relaxed),
            self.version.load(),
            i32::from(packet_id),
            payload,
        );
        Some((server, event))
    }

    fn finish_connection_sent_event(event: &BedrockConnectionPacketSentEvent) -> TranslatedPacket {
        let extras = event
            .extra_packets
            .iter()
            .filter_map(|(id, payload)| frame_raw_packet(*id, payload))
            .collect();
        if event.cancelled {
            return (None, extras);
        }
        (frame_raw_packet(event.packet_id, &event.payload), extras)
    }

    fn translate_outgoing(&self, packet_data: Bytes, observe: bool) -> TranslatedPacket {
        if let Some((server, mut event, current_version)) =
            self.prepare_play_sent_event(&packet_data, observe)
        {
            server.plugin_manager.fire_blocking(&server, &mut event);
            return Self::finish_play_sent_event(packet_data, &event, current_version);
        }
        if let Some((server, mut event)) = self.prepare_connection_sent_event(&packet_data) {
            server.plugin_manager.fire_blocking(&server, &mut event);
            return Self::finish_connection_sent_event(&event);
        }
        (Some(packet_data), Vec::new())
    }

    async fn translate_outgoing_async(
        &self,
        packet_data: Bytes,
        observe: bool,
    ) -> TranslatedPacket {
        if let Some((server, mut event, current_version)) =
            self.prepare_play_sent_event(&packet_data, observe)
        {
            server.plugin_manager.fire(&server, &mut event).await;
            return Self::finish_play_sent_event(packet_data, &event, current_version);
        }
        if let Some((server, mut event)) = self.prepare_connection_sent_event(&packet_data) {
            server.plugin_manager.fire(&server, &mut event).await;
            return Self::finish_connection_sent_event(&event);
        }
        (Some(packet_data), Vec::new())
    }

    fn reserve_bytes(&self, packet_len: usize) -> Option<usize> {
        if self.is_closed() {
            return None;
        }
        let prev_bytes = self.pending_bytes.fetch_add(packet_len, Ordering::AcqRel);
        let new_bytes = prev_bytes.saturating_add(packet_len);

        if new_bytes > MAX_PENDING_BYTES {
            decrement_pending_bytes(&self.pending_bytes, packet_len);
            if !self.is_closed() {
                warn!(
                    "Bedrock client {} outbound packet buffer overflow ({} bytes > {} bytes). Closing connection.",
                    self.address, new_bytes, MAX_PENDING_BYTES
                );
                self.close_token.cancel();
            }
            return None;
        }
        Some(packet_len)
    }

    fn queue_outgoing(&self, packet: OutgoingPacket, packet_len: usize) -> bool {
        if let Err(err) = self.outgoing_packet_queue_send.send(packet) {
            decrement_pending_bytes(&self.pending_bytes, packet_len);
            if !self.is_closed() {
                error!("Failed to add packet to the outgoing packet queue for client: {err}");
            }
            return false;
        }
        true
    }

    fn queue_extra_packets(&self, extras: Vec<Bytes>) -> bool {
        for extra in extras {
            let Some(packet_len) = self.reserve_bytes(extra.len()) else {
                return false;
            };
            if !self.queue_outgoing(OutgoingPacket::normal(extra), packet_len) {
                return false;
            }
        }
        !self.is_closed()
    }

    fn queue_translated_packet(&self, packet_data: Bytes, extras: Vec<Bytes>) -> bool {
        let Some(packet_len) = self.reserve_bytes(packet_data.len()) else {
            return false;
        };
        if !self.queue_outgoing(OutgoingPacket::normal(packet_data), packet_len) {
            return false;
        }
        self.queue_extra_packets(extras);
        true
    }

    pub async fn enqueue_packet(&self, packet_data: Bytes) {
        self.enqueue_packet_data(packet_data).await;
    }

    pub fn try_enqueue_packet(&self, packet_data: Bytes) {
        self.try_enqueue_packet_data(packet_data);
    }

    /// Queues a clientbound packet to be sent to the connected client.
    pub async fn enqueue_packet_data(&self, packet_data: Bytes) -> bool {
        if self.is_closed() {
            return false;
        }
        let (packet_data, extras) = self.translate_outgoing_async(packet_data, true).await;
        match packet_data {
            Some(packet_data) => self.queue_translated_packet(packet_data, extras),
            None => self.queue_extra_packets(extras),
        }
    }

    /// `false` when the packet was dropped instead of queued.
    pub fn try_enqueue_packet_data(&self, packet_data: Bytes) -> bool {
        self.try_enqueue_packet_data_inner(packet_data, true)
    }

    /// Like [`Self::try_enqueue_packet_data`] without firing `PacketSentEvent` on current-version
    /// clients, while still running version translation for legacy clients.
    pub fn try_enqueue_packet_data_unobserved(&self, packet_data: Bytes) -> bool {
        self.try_enqueue_packet_data_inner(packet_data, false)
    }

    /// Queues an already-formatted raw Bedrock packet (`packet_id` + client-version `payload`)
    /// directly onto the wire queue without firing `PacketSentEvent`.
    pub fn try_enqueue_raw_packet_direct(&self, packet_id: u16, payload: &[u8]) -> bool {
        let Some(packet_data) = frame_raw_packet(i32::from(packet_id), payload) else {
            return false;
        };
        let Some(packet_len) = self.reserve_bytes(packet_data.len()) else {
            return false;
        };
        self.queue_outgoing(OutgoingPacket::normal(packet_data), packet_len)
    }

    fn try_enqueue_packet_data_inner(&self, packet_data: Bytes, observe: bool) -> bool {
        if self.is_closed() {
            return false;
        }
        let (packet_data, extras) = self.translate_outgoing(packet_data, observe);
        match packet_data {
            Some(packet_data) => self.queue_translated_packet(packet_data, extras),
            None => self.queue_extra_packets(extras),
        }
    }

    pub fn write_raw_packet<P: BClientPacket>(
        packet: &P,
        mut writer: impl Write,
    ) -> Result<(), Error> {
        VarUInt(P::PACKET_ID as u32).write(&mut writer)?;
        packet.write_packet(writer)
    }

    pub async fn write_game_packet<P: BClientPacket>(
        &self,
        packet: &P,
        write: impl Write,
    ) -> Result<(), Error> {
        let mut packet_payload = Vec::new();
        packet.write_packet(&mut packet_payload)?;

        let encoder = self.network_writer.read().await;
        encoder.write_game_packet(
            P::PACKET_ID as u16,
            SubClient::Main,
            SubClient::Main,
            &packet_payload,
            write,
        )
    }

    pub fn serialize_packet<P: BClientPacket>(&self, packet: &P) -> Result<Bytes, Error> {
        let mut buf = Vec::new();
        Self::write_raw_packet(packet, &mut buf)?;
        Ok(buf.into())
    }

    pub async fn send_packet<P: BClientPacket>(&self, packet: &P) {
        match self.serialize_packet(packet) {
            Ok(data) => self.send_game_packet(data).await,
            Err(err) => error!("Failed to serialize Bedrock packet: {err}"),
        }
    }

    pub async fn enqueue_client_packet<P: BClientPacket>(&self, packet: &P) {
        match self.serialize_packet(packet) {
            Ok(data) => self.enqueue_packet(data).await,
            Err(err) => error!("Failed to serialize Bedrock packet: {err}"),
        }
    }

    pub fn try_enqueue_client_packet<P: BClientPacket>(&self, packet: &P) {
        match self.serialize_packet(packet) {
            Ok(data) => self.try_enqueue_packet(data),
            Err(err) => error!("Failed to serialize Bedrock packet: {err}"),
        }
    }

    async fn send_and_wait_translated(&self, (packet_data, extras): (Option<Bytes>, Vec<Bytes>)) {
        if self.is_closed() {
            return;
        }
        let mut packets = Vec::with_capacity(usize::from(packet_data.is_some()) + extras.len());
        if let Some(packet_data) = packet_data {
            packets.push(packet_data);
        }
        packets.extend(extras);
        let Some(last_packet) = packets.pop() else {
            return;
        };
        for packet in packets {
            let Some(packet_len) = self.reserve_bytes(packet.len()) else {
                return;
            };
            if !self.queue_outgoing(OutgoingPacket::normal(packet), packet_len) {
                return;
            }
        }
        let Some(last_len) = self.reserve_bytes(last_packet.len()) else {
            return;
        };
        let (tx, rx) = oneshot::channel();
        if self.queue_outgoing(OutgoingPacket::priority(last_packet, tx), last_len) {
            let _ = rx.await;
        }
    }

    pub async fn send_game_packet(&self, packet_data: Bytes) {
        if self.is_closed() {
            return;
        }
        let translated = self.translate_outgoing_async(packet_data, true).await;
        self.send_and_wait_translated(translated).await;
    }

    pub async fn close(&self) {
        if self.close_token.is_cancelled() {
            return;
        }
        self.close_token.cancel();
        self.session.close().await;
        self.be_clients.lock().await.remove(&self.address);
    }

    pub async fn await_tasks(&self) {
        self.tasks.close();
        self.tasks.wait().await;
    }

    pub fn is_closed(&self) -> bool {
        self.close_token.is_cancelled() || self.session.is_closed()
    }

    pub fn enqueue_spawn_packet(&self, entity: &dyn crate::entity::EntityBase) {
        entity.send_bedrock_spawn_packet(self);
    }

    async fn process_batch(
        self: &Arc<Self>,
        server: &Arc<Server>,
        payload: Vec<u8>,
    ) -> Result<(), Error> {
        let decompressed_payload = self
            .get_packet_payload(payload)
            .await
            .ok_or_else(|| Error::other("Failed to decompress game packet batch"))?;
        let mut cursor = Cursor::new(decompressed_payload);

        while (cursor.position() as usize) < cursor.get_ref().len() {
            let game_packet = self
                .network_reader
                .lock()
                .await
                .get_game_packet(&mut cursor)
                .map_err(|e| Error::other(e.to_string()))?;

            if !self.packet_limiter.check_packet() {
                warn!(
                    "Bedrock client {} exceeded packet rate limit (rate: {}/s)",
                    self.address,
                    self.packet_limiter.max_rate()
                );
                self.kick(
                    DisconnectReason::Kicked,
                    server
                        .advanced_config
                        .networking
                        .bedrock
                        .packet_limiter
                        .kick_message
                        .clone(),
                )
                .await;
                return Err(Error::other("Packet rate limit exceeded"));
            }

            self.handle_game_packet(game_packet).await?;
        }

        Ok(())
    }

    async fn handle_game_packet(&self, packet: RawPacket) -> Result<(), Error> {
        if let Err(err) = self.incoming_game_packet_send.send(packet).await {
            debug!("Failed to send game packet to session task: {err}");
        }
        Ok(())
    }

    async fn translate_incoming_login(
        &self,
        server: &Arc<Server>,
        packet: RawPacket,
    ) -> Option<RawPacket> {
        if self.is_current_protocol()
            || !server
                .plugin_manager
                .has_handlers::<BedrockConnectionPacketReceivedEvent>()
        {
            return Some(packet);
        }
        let mut event = BedrockConnectionPacketReceivedEvent::new(
            self.id,
            self.protocol_version.load(Ordering::Relaxed),
            self.version.load(),
            packet.id,
            packet.payload,
        );
        server.plugin_manager.fire(server, &mut event).await;
        (!event.cancelled).then_some(RawPacket {
            id: event.packet_id,
            payload: event.payload,
        })
    }

    pub async fn handle_login_sequence(
        self: &Arc<Self>,
        server: &Arc<Server>,
    ) -> PacketHandlerResult {
        while let Some(packet) = self.get_packet().await {
            if packet.id == SRequestNetworkSettings::PACKET_ID {
                let payload = &mut Cursor::new(&packet.payload);
                let packet = match SRequestNetworkSettings::read(payload) {
                    Ok(p) => p,
                    Err(err) => {
                        error!("Failed to read SRequestNetworkSettings: {err}");
                        continue;
                    }
                };
                if !self.handle_request_network_settings(packet, server).await {
                    return PacketHandlerResult::Stop;
                }
                continue;
            }

            // Pre-1.19.30 clients send SLogin directly without SRequestNetworkSettings.
            if self.protocol_version.load(Ordering::Relaxed) == 0
                && packet.id == SLogin::PACKET_ID
                && let Some(bytes) = packet.payload.get(..4)
                && let Ok(bytes) = <[u8; 4]>::try_from(bytes)
            {
                let client_protocol = i32::from_be_bytes(bytes);
                if !self
                    .admit_legacy_login_protocol(client_protocol, server)
                    .await
                {
                    return PacketHandlerResult::Stop;
                }
            }

            let Some(packet) = self.translate_incoming_login(server, packet).await else {
                continue;
            };
            let payload = &mut Cursor::new(&packet.payload);
            match packet.id {
                SLogin::PACKET_ID => {
                    let packet = match SLogin::read(payload) {
                        Ok(p) => p,
                        Err(err) => {
                            error!("Failed to read SLogin: {err}");
                            self.kick(DisconnectReason::BadPacket, err.to_string())
                                .await;
                            return PacketHandlerResult::Stop;
                        }
                    };
                    match self.handle_login(packet, server).await {
                        Ok(result) => return result,
                        Err(err) => {
                            self.kick(DisconnectReason::Unknown, err.to_string()).await;
                            return PacketHandlerResult::Stop;
                        }
                    }
                }
                _ => {
                    debug!(
                        "Received unexpected game packet {} during login sequence",
                        packet.id
                    );
                }
            }
        }
        PacketHandlerResult::Stop
    }

    pub async fn progress_player_packets(self: &Arc<Self>, player: &Arc<Player>) {
        while let Some(packet) = self.get_packet().await {
            player.inbound_packets.push(packet);
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn handle_play_packet(
        self: &Arc<Self>,
        player: &Arc<Player>,
        server: &Arc<Server>,
        packet: &RawPacket,
    ) -> Result<(), Error> {
        let payload = &packet.payload[..];
        let reader = &mut &payload[..];
        match packet.id {
            SClientCacheStatus::PACKET_ID => {
                let packet = SClientCacheStatus::read(reader)?;
                self.client_cache_supported
                    .store(packet.is_cache_supported, Ordering::Relaxed);
            }
            SClientCacheBlobStatus::PACKET_ID => {
                let packet = SClientCacheBlobStatus::read(reader)?;
                self.handle_client_cache_blob_status(packet);
            }
            SResourcePackClientResponse::PACKET_ID => {
                let packet = SResourcePackClientResponse::read(reader)?;
                let client = self.clone();
                let server_c = server.clone();
                server.spawn_task(async move {
                    client
                        .handle_resource_pack_response(packet, &server_c)
                        .await;
                });
            }
            SResourcePackChunkRequest::PACKET_ID => {
                let packet = SResourcePackChunkRequest::read(reader)?;
                let client = self.clone();
                let server_c = server.clone();
                server.spawn_task(async move {
                    client
                        .handle_resource_pack_chunk_request(packet, &server_c)
                        .await;
                });
            }
            SPlayerAuthInput::PACKET_ID => {
                let packet = SPlayerAuthInput::read(reader)?;
                self.handle_player_auth_input(player, packet, server);
            }
            SRequestChunkRadius::PACKET_ID => {
                let packet = SRequestChunkRadius::read(reader)?;
                self.handle_request_chunk_radius(player, &packet);
            }
            SInventoryTransaction::PACKET_ID => {
                let packet = SInventoryTransaction::read(reader)?;
                self.handle_inventory_action(player, packet);
            }
            SItemStackRequest::PACKET_ID => {
                let packet = SItemStackRequest::read(reader)?;
                self.handle_item_stack_request(player, packet);
            }
            SInteract::PACKET_ID => {
                let packet = SInteract::read(reader)?;
                self.handle_interaction(player, &packet);
            }
            SContainerClose::PACKET_ID => {
                let packet = SContainerClose::read(reader)?;
                self.handle_container_close(player, &packet);
            }
            SText::PACKET_ID => {
                let text = SText::read(reader)?;
                let client = self.clone();
                let player_c = player.clone();
                let server_c = server.clone();
                player.spawn_task(async move {
                    client.handle_chat_message(&server_c, &player_c, text).await;
                });
            }
            SCommandRequest::PACKET_ID => {
                let req = SCommandRequest::read(reader)?;
                let client = self.clone();
                let player_c = player.clone();
                let server_c = server.clone();
                player.spawn_task(async move {
                    client.handle_chat_command(&player_c, &server_c, req).await;
                });
            }
            SSetLocalPlayerAsInitialized::PACKET_ID => {
                self.handle_set_local_player_as_initialized(
                    player,
                    &SSetLocalPlayerAsInitialized::read(reader)?,
                );
            }
            SSetPlayerInventoryOptions::PACKET_ID => {
                let _ = SSetPlayerInventoryOptions::read(reader)?;
                // Ignore for now
            }
            SCursorItemDrag::PACKET_ID => {
                let _ = SCursorItemDrag::read(reader)?;
                // Ignore for now
            }
            SStonecutterSetRecipe::PACKET_ID => {
                let packet = SStonecutterSetRecipe::read(reader)?;
                self.handle_stonecutter_set_recipe(player, &packet);
            }
            SMatchmakingCancel::PACKET_ID => {
                let _ = SMatchmakingCancel::read(reader)?;
                // Ignore for now
            }
            SRegisterAudioContent::PACKET_ID => {
                let _ = SRegisterAudioContent::read(reader)?;
                // Ignore for now
            }
            SPlayerAction::PACKET_ID => {
                let packet = SPlayerAction::read(reader)?;
                self.handle_player_action(player, server, &packet);
            }
            SRespawn::PACKET_ID => {
                let packet = SRespawn::read(reader)?;
                self.handle_respawn(player, &packet);
            }
            SAnimate::PACKET_ID => {
                self.handle_animate(player, &SAnimate::read(reader)?);
            }
            SActorEvent::PACKET_ID => {
                self.handle_actor_event(player, &SActorEvent::read(reader)?);
            }
            SEmote::PACKET_ID => {
                self.handle_emote(player, SEmote::read_slice(reader)?);
            }
            SEmoteList::PACKET_ID => {
                self.handle_emote_list(player, &SEmoteList::read(reader)?);
            }
            SModalFormResponse::PACKET_ID => {
                let form_resp = SModalFormResponse::read(reader)?;
                self.handle_modal_form_response(player, server, form_resp);
            }
            // Ignore for now; movement is client authoritative, so there is no prediction to sync
            SLoadingScreen::PACKET_ID | SClientMovementPredictionSync::PACKET_ID => {}
            SBlockPickRequest::PACKET_ID => {
                let packet = SBlockPickRequest::read(reader)?;
                self.handle_block_pick_request(player, &packet);
            }
            SBlockActorData::PACKET_ID => {
                let packet = SBlockActorData::read(reader)?;
                self.handle_block_actor_data(player, &packet);
            }
            SRequestAbility::PACKET_ID => {
                self.handle_request_ability(player, &SRequestAbility::read(reader)?);
            }
            NetworkStackLatency::PACKET_ID => {
                self.handle_network_stack_latency(player, NetworkStackLatency::read(reader)?);
            }
            SMobEquipment::PACKET_ID => {
                let packet = SMobEquipment::read(reader)?;
                self.handle_mob_equipment(server, player, &packet);
            }
            SPlayerHotbar::PACKET_ID => {
                let packet = SPlayerHotbar::read(reader)?;
                self.handle_player_hotbar(server, player, &packet);
            }
            SPacketViolationWarning::PACKET_ID => {
                let warning = SPacketViolationWarning::read(reader)?;
                warn!(
                    violation_type = warning.violation_type.0,
                    violation_severity = warning.violation_severity.0,
                    violation_packet_id = warning.violation_packet_id.0,
                    violation_context = %warning.violation_context,
                    "Bedrock client rejected a server packet"
                );
            }
            _ => {
                warn!("Bedrock: Received Unknown Game packet: {}", packet.id);
            }
        }
        Ok(())
    }

    pub fn handle_client_cache_blob_status(&self, packet: SClientCacheBlobStatus) {
        let missing_blobs = {
            let mut cache = self
                .blob_cache
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for hash in packet.hit_hashes {
                cache.remove(hash);
            }
            let mut missing_blobs = Vec::with_capacity(packet.miss_hashes.len());
            for hash in packet.miss_hashes {
                if let Some(blob) = cache.get(hash) {
                    missing_blobs.push(MissingBlobData {
                        blob_id: hash,
                        blob_data: blob.clone(),
                    });
                    cache.mark_answered(hash);
                } else {
                    warn!("Client requested missing blob {hash:#x} not found in server cache");
                }
            }
            missing_blobs
        };
        if !missing_blobs.is_empty() {
            self.try_enqueue_client_packet(&CClientCacheMissResponse { missing_blobs });
        }
    }

    pub async fn await_close_interrupt(&self) {
        self.close_token.cancelled().await;
    }

    pub async fn get_packet_payload(&self, packet: Vec<u8>) -> Option<Vec<u8>> {
        let mut network_reader = self.network_reader.lock().await;
        tokio::select! {
            () = self.await_close_interrupt() => {
                debug!("Canceling player packet processing");
                None
            },
            packet_result = network_reader.get_packet_payload(packet) => {
                match packet_result {
                    Ok(packet) => Some(packet),
                    Err(err) => {
                        if !matches!(err, PacketDecodeError::ConnectionClosed) {
                            debug!("Failed to decode packet from client: {err}");
                            let text = format!("Error while reading incoming packet {err}");
                            self.kick(DisconnectReason::BadPacket, text).await;
                        }
                        None
                    }
                }
            }
        }
    }

    pub fn spawn_task<F>(&self, task: F) -> Option<JoinHandle<F::Output>>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        if self.close_token.is_cancelled() {
            None
        } else {
            Some(self.tasks.spawn_on(task, &self.rt_handle))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_protocol::bedrock::client::biome_definition_list::CBiomeDefinitionList;

    #[test]
    fn raw_packet_framing_roundtrips_high_and_0xfe_packet_ids() {
        for id in [1i32, 143, 254, 300, 416, 1023] {
            let payload = [0xAA, 0xBB, 0xCC];
            let framed = frame_raw_packet(id, &payload).expect("valid packet id");
            let (decoded_id, decoded_payload) =
                split_raw_packet(&framed).expect("split framed packet");
            assert_eq!(i32::from(decoded_id), id);
            assert_eq!(&decoded_payload[..], &payload[..]);
        }
        assert!(frame_raw_packet(-1, &[]).is_none());
        assert!(frame_raw_packet(i32::from(u16::MAX) + 1, &[]).is_none());
    }

    #[test]
    fn write_raw_packet_preserves_multi_byte_packet_ids() {
        let mut buf = Vec::new();
        BedrockClient::write_raw_packet(&CBiomeDefinitionList, &mut buf).unwrap();
        let bytes = Bytes::from(buf);
        let (packet_id, _) = split_raw_packet(&bytes).expect("split CBiomeDefinitionList");
        assert_eq!(i32::from(packet_id), CBiomeDefinitionList::PACKET_ID);
    }
}
