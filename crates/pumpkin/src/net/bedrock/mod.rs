pub mod nethernet;
pub mod play;
use crossbeam::atomic::AtomicCell;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    io::{Cursor, Error, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU16, AtomicU32, AtomicUsize, Ordering},
    },
    time::UNIX_EPOCH,
};

use tracing::{debug, error, warn};

use bytes::Bytes;
use pumpkin_config::{networking::bedrock::RakNetStatus, networking::compression::CompressionInfo};
use pumpkin_protocol::{
    BClientPacket, PacketDecodeError, RawPacket,
    bedrock::{
        BEDROCK_GAME_PACKET, MTU, RAKNET_ACK, RAKNET_MAGIC, RAKNET_NACK, RAKNET_VALID,
        RakReliability, SPLIT_FRAME_MAX_CONTENT, SubClient, UDP_HEADER_SIZE,
        ack::Acknowledge,
        client::{
            client_cache_miss_response::{CClientCacheMissResponse, MissingBlobData},
            disconnect::CDisconnect,
            raknet::connection::{
                CAlreadyConnected, CConnectionBanned, CConnectionRequestAccepted,
                CDisconnect as CRakDisconnect, CNoFreeIncomingConnections,
            },
        },
        frame_set::{Frame, FrameSet},
        packet_decoder::{BedrockBatchDecoder, EncryptionAlreadyEnabledError},
        packet_encoder::BedrockBatchEncoder,
        server::{
            actor_event::SActorEvent,
            animate::SAnimate,
            block_pick_request::SBlockPickRequest,
            client_cache_blob_status::SClientCacheBlobStatus,
            client_cache_status::SClientCacheStatus,
            client_to_server_handshake::SClientToServerHandshake,
            command_request::SCommandRequest,
            container_close::SContainerClose,
            emote::SEmote,
            emote_list::SEmoteList,
            interact::SInteract,
            inventory_transaction::SInventoryTransaction,
            loading_screen::SLoadingScreen,
            login::SLogin,
            mob_equipment::SMobEquipment,
            packet_violation_warning::SPacketViolationWarning,
            player_action::SPlayerAction,
            player_auth_input::SPlayerAuthInput,
            raknet::{
                connection::{
                    SConnectedPing, SConnectionLost, SConnectionRequest, SDisconnect,
                    SNewIncomingConnection,
                },
                open_connection::{SOpenConnectionRequest1, SOpenConnectionRequest2},
                unconnected_ping::{SUnconnectedPing, SUnconnectedPingOpenConnections},
            },
            request_ability::SRequestAbility,
            request_chunk_radius::SRequestChunkRadius,
            request_network_settings::SRequestNetworkSettings,
            resource_pack_client_response::SResourcePackClientResponse,
            respawn::SRespawn,
            set_local_player_as_initialized::SSetLocalPlayerAsInitialized,
            set_player_inventory_options::SSetPlayerInventoryOptions,
            text::SText,
        },
    },
    codec::u24,
    packet::Packet,
    serial::{PacketRead, PacketReadSlice},
};
use tokio::{
    net::UdpSocket,
    sync::mpsc::{Receiver, Sender, UnboundedReceiver, UnboundedSender, error::TryRecvError},
    sync::{Mutex, RwLock, oneshot},
    task::JoinHandle,
};

use tokio_util::{sync::CancellationToken, task::TaskTracker};

pub mod connection;
pub mod level_chunk;
pub mod login;
pub mod open_connection;
pub mod status;
pub mod unconnected;
use self::level_chunk::CLevelChunk;
use self::nethernet::NetherNetSession;
use crate::{
    entity::player::Player,
    net::{
        DisconnectReason, GameProfile, MAX_PENDING_BYTES, PacketHandlerResult, PacketRateLimiter,
        PlayerConfig, decrement_pending_bytes,
    },
    plugin::api::events::world::chunk_send::ChunkSend,
    server::Server,
};

const MAX_INBOUND_SPLIT_SIZE: u32 = 4096;
const MAX_ORDERED_QUEUE_SIZE: usize = 256;
use arc_swap::ArcSwap;
use pumpkin_protocol::bedrock::server::login::ClientData;
use pumpkin_util::version::BedrockMinecraftVersion;
use pumpkin_world::level::SyncChunk;

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

/// Distinguishes `RakNet` datagrams from ICE traffic on the shared UDP port.
#[must_use]
pub fn is_raknet_packet(packet: &[u8]) -> bool {
    let Some(&id) = packet.first() else {
        return false;
    };
    if id & RAKNET_VALID != 0 {
        return true;
    }
    match id {
        id if id == SUnconnectedPing::PACKET_ID as u8
            || id == SUnconnectedPingOpenConnections::PACKET_ID as u8 =>
        {
            packet.get(9..25) == Some(RAKNET_MAGIC.as_slice())
        }
        id if id == SOpenConnectionRequest1::PACKET_ID as u8
            || id == SOpenConnectionRequest2::PACKET_ID as u8 =>
        {
            packet.get(1..17) == Some(RAKNET_MAGIC.as_slice())
        }
        _ => false,
    }
}

/// Whether a `RakNet` server-list ping is answered for `ip`.
#[must_use]
const fn advertises_raknet(status: RakNetStatus, ip: IpAddr) -> bool {
    match status {
        RakNetStatus::Always => true,
        RakNetStatus::Remote => !is_local_address(ip),
        RakNetStatus::Never => false,
    }
}

/// Whether `ip` belongs to a local or private network.
#[must_use]
pub(crate) const fn is_local_address(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => ip.is_private() || ip.is_loopback() || ip.is_link_local(),
        IpAddr::V6(ip) => ip.is_loopback() || ip.is_unique_local(),
    }
}

pub enum BedrockTransport {
    RakNet(Arc<UdpSocket>),
    NetherNet(Arc<NetherNetSession>),
}

pub struct BedrockClient {
    transport: BedrockTransport,
    /// The client's IP address.
    pub address: SocketAddr,
    pub player: ArcSwap<Option<Arc<Player>>>,
    pub version: AtomicCell<BedrockMinecraftVersion>,
    pub client_data: ArcSwap<Option<Arc<ClientData>>>,
    /// All Bedrock clients
    /// This list is used to remove the client if the connection gets closed
    pub be_clients: Arc<Mutex<HashMap<SocketAddr, Arc<Self>>>>,

    tasks: TaskTracker,
    rt_handle: tokio::runtime::Handle,
    outgoing_packet_queue_send: UnboundedSender<OutgoingPacket>,
    /// A queue of serialized packets to send to the network
    outgoing_packet_queue_recv: Mutex<Option<UnboundedReceiver<OutgoingPacket>>>,

    outgoing_packet_priority_send: UnboundedSender<OutgoingPacket>,
    outgoing_packet_priority_recv: Mutex<Option<UnboundedReceiver<OutgoingPacket>>>,

    /// Tracks total buffered payload bytes in the outgoing queues.
    pub pending_bytes: Arc<AtomicUsize>,

    /// The packet encoder for outgoing packets.
    network_writer: Arc<RwLock<BedrockBatchEncoder>>,
    /// The packet decoder for incoming packets.
    network_reader: Mutex<BedrockBatchDecoder>,

    /// `RakNet` outgoing framing state. Unused by `NetherNet`, which leaves framing to WebRTC.
    output_sequence_number: AtomicU32,
    output_reliable_number: AtomicU32,
    output_split_number: AtomicU16,
    output_ordered_index: AtomicU32,
    /// `RakNet` incoming reassembly state.
    compounds: Mutex<HashMap<u16, Vec<Option<Frame>>>>,
    received_sequences: Mutex<HashSet<u32>>,
    pending_acks: Mutex<Vec<u32>>,
    /// Framesets waiting for an acknowledgement, kept for retransmission.
    #[allow(clippy::type_complexity)]
    unacked_outgoing_frames: Mutex<HashMap<u32, (u8, Vec<u8>, std::time::Instant)>>,
    expected_order_index: Mutex<HashMap<u8, u32>>,
    highest_sequence_index: Mutex<HashMap<u8, u32>>,
    ordered_queues: Mutex<HashMap<u8, BTreeMap<u32, Frame>>>,

    /// The next form ID to use for custom forms.
    pub next_form_id: AtomicU32,
    pub inventory_opened: AtomicBool,
    /// Separate from normal vitals caching so the first rejected use always gets corrected.
    last_food_rejection_tick: AtomicCell<Option<i32>>,
    pub client_cache_supported: AtomicBool,
    pub blob_cache: std::sync::Mutex<HashMap<u64, Vec<u8>>>,
    /// An notifier that is triggered when this client is closed.
    close_token: CancellationToken,
    last_seen: Arc<AtomicCell<std::time::Instant>>,
    incoming_game_packet_send: Sender<RawPacket>,
    incoming_game_packet_recv: Mutex<Option<Receiver<RawPacket>>>,
    /// The login result of a `RakNet` client that is still completing the encryption handshake.
    pending_profile: ArcSwap<Option<Arc<(GameProfile, PlayerConfig)>>>,
    /// Packet rate limiter for incoming client packets.
    pub packet_limiter: PacketRateLimiter,
}

impl BedrockClient {
    /// Creates a client speaking `RakNet` over the shared Bedrock UDP socket.
    #[must_use]
    pub fn new(
        socket: Arc<UdpSocket>,
        address: SocketAddr,
        be_clients: Arc<Mutex<HashMap<SocketAddr, Arc<Self>>>>,
        packet_limiter: PacketRateLimiter,
    ) -> Self {
        Self::with_transport(
            BedrockTransport::RakNet(socket),
            address,
            be_clients,
            packet_limiter,
        )
    }

    /// Creates a client carried by a `NetherNet` WebRTC session.
    #[must_use]
    pub fn new_nethernet(
        session: Arc<NetherNetSession>,
        address: SocketAddr,
        be_clients: Arc<Mutex<HashMap<SocketAddr, Arc<Self>>>>,
        packet_limiter: PacketRateLimiter,
    ) -> Self {
        Self::with_transport(
            BedrockTransport::NetherNet(session),
            address,
            be_clients,
            packet_limiter,
        )
    }

    fn with_transport(
        transport: BedrockTransport,
        address: SocketAddr,
        be_clients: Arc<Mutex<HashMap<SocketAddr, Arc<Self>>>>,
        packet_limiter: PacketRateLimiter,
    ) -> Self {
        let (send, recv) = tokio::sync::mpsc::unbounded_channel();
        let (priority_send, priority_recv) = tokio::sync::mpsc::unbounded_channel();
        let (incoming_send, incoming_recv) = tokio::sync::mpsc::channel(4096);
        let rt_handle = tokio::runtime::Handle::current();
        Self {
            transport,
            player: ArcSwap::new(Arc::new(None)),
            address,
            version: AtomicCell::new(BedrockMinecraftVersion::Unknown),
            client_data: ArcSwap::new(Arc::new(None)),
            be_clients,
            network_writer: Arc::new(RwLock::new(BedrockBatchEncoder::new())),
            network_reader: Mutex::new(BedrockBatchDecoder::new()),
            tasks: TaskTracker::new(),
            rt_handle,
            outgoing_packet_queue_send: send,
            outgoing_packet_queue_recv: Mutex::new(Some(recv)),
            outgoing_packet_priority_send: priority_send,
            outgoing_packet_priority_recv: Mutex::new(Some(priority_recv)),
            pending_bytes: Arc::new(AtomicUsize::new(0)),
            output_sequence_number: AtomicU32::new(0),
            output_reliable_number: AtomicU32::new(0),
            output_split_number: AtomicU16::new(0),
            output_ordered_index: AtomicU32::new(0),
            compounds: Mutex::new(HashMap::new()),
            received_sequences: Mutex::new(HashSet::new()),
            pending_acks: Mutex::new(Vec::new()),
            unacked_outgoing_frames: Mutex::new(HashMap::new()),
            expected_order_index: Mutex::new(HashMap::new()),
            highest_sequence_index: Mutex::new(HashMap::new()),
            ordered_queues: Mutex::new(HashMap::new()),
            next_form_id: AtomicU32::new(0),
            inventory_opened: AtomicBool::new(false),
            last_food_rejection_tick: AtomicCell::new(None),
            client_cache_supported: AtomicBool::new(false),
            blob_cache: std::sync::Mutex::new(HashMap::new()),
            close_token: CancellationToken::new(),
            last_seen: Arc::new(AtomicCell::new(std::time::Instant::now())),
            incoming_game_packet_send: incoming_send,
            incoming_game_packet_recv: Mutex::new(Some(incoming_recv)),
            pending_profile: ArcSwap::new(Arc::new(None)),
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
            let Some(mut priority_packet_receiver) =
                client.outgoing_packet_priority_recv.lock().await.take()
            else {
                return;
            };
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(100));

            loop {
                let recv_result = tokio::select! {
                    biased;
                    res = priority_packet_receiver.recv() => res,
                    res = packet_receiver.recv() => res,
                    _ = interval.tick() => {
                        if !client.tick_connection().await {
                            break;
                        }
                        continue;
                    }
                    () = client.close_token.cancelled() => {
                        priority_packet_receiver
                            .try_recv()
                            .ok()
                            .or_else(|| packet_receiver.try_recv().ok())
                    }
                };

                let Some(packet) = recv_result else {
                    break;
                };

                let mut packet_batch = Vec::with_capacity(MAX_BATCH_SIZE);
                packet_batch.push(packet);

                while packet_batch.len() < MAX_BATCH_SIZE {
                    match priority_packet_receiver.try_recv() {
                        Ok(packet) => {
                            packet_batch.push(packet);
                            continue;
                        }
                        Err(TryRecvError::Disconnected | TryRecvError::Empty) => {}
                    }

                    match packet_receiver.try_recv() {
                        Ok(packet) => packet_batch.push(packet),
                        Err(TryRecvError::Disconnected | TryRecvError::Empty) => break,
                    }
                }

                for packet in packet_batch {
                    let packet_len = packet.data.len();

                    match &client.transport {
                        BedrockTransport::RakNet(_) => {
                            // Minecraft's own layer encrypts everything after the `0xfe` batch
                            // marker, so the frameset header itself stays readable on the wire.
                            let mut framed = packet.data.to_vec();
                            if framed.first() == Some(&BEDROCK_GAME_PACKET) {
                                let mut payload = framed[1..].to_vec();
                                client.network_writer.read().await.encrypt(&mut payload);
                                framed.truncate(1);
                                framed.extend_from_slice(&payload);
                            }

                            client
                                .send_framed_packet_data(framed, RakReliability::ReliableOrdered)
                                .await;
                        }
                        BedrockTransport::NetherNet(session) => {
                            let data = packet.data.strip_prefix(&[BEDROCK_GAME_PACKET]);
                            let Some(data) = data else {
                                warn!("Refusing to send a non-game packet over NetherNet");
                                decrement_pending_bytes(&client.pending_bytes, packet_len);
                                continue;
                            };
                            if let Err(error) = session.send(Bytes::copy_from_slice(data)).await {
                                warn!(
                                    "Failed to send NetherNet packet to {}: {error}",
                                    client.address
                                );
                                decrement_pending_bytes(&client.pending_bytes, packet_len);
                                client.close().await;
                                return;
                            }
                        }
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

        if self.raknet_socket().is_none() {
            return true;
        }

        let mut pending = self.pending_acks.lock().await;
        if !pending.is_empty() {
            let ack = Acknowledge::new(pending.clone());
            pending.clear();
            let _ = self.send_acknowledgement(&ack, RAKNET_ACK).await;
        }
        drop(pending);

        let now = std::time::Instant::now();
        let mut resend = Vec::new();
        let mut unacked = self.unacked_outgoing_frames.lock().await;
        for (sequence, (id, data, timestamp)) in unacked.iter_mut() {
            if now.duration_since(*timestamp) > std::time::Duration::from_secs(1) {
                resend.push((*sequence, *id, data.clone()));
                *timestamp = now;
                if resend.len() >= 50 {
                    break;
                }
            }
        }
        drop(unacked);

        if resend.is_empty() {
            return true;
        }
        for (sequence, id, data) in resend {
            debug!("Resending reliable sequence {sequence} (ID: {id})");
            if let Err(error) = self.send_datagram(&data).await {
                warn!("Failed to resend packet for sequence {sequence}: {error}");
            }
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

    /// Handles one `RakNet` datagram received on the shared Bedrock UDP socket.
    pub async fn process_packet(self: &Arc<Self>, server: &Arc<Server>, packet: Bytes) {
        self.last_seen.store(std::time::Instant::now());
        if let Err(error) = self.handle_packet_payload(server, packet).await {
            error!(
                "Failed to handle packet payload for {}: {error}",
                self.address
            );
            self.kick(DisconnectReason::BadPacket, error.to_string())
                .await;
        }
    }

    /// Whether this client is carried by `NetherNet` rather than `RakNet`.
    #[must_use]
    pub const fn is_nethernet(&self) -> bool {
        matches!(self.transport, BedrockTransport::NetherNet(_))
    }

    pub fn nethernet_public_key(&self) -> Option<&pumpkin_auth::p384::PublicKey> {
        match &self.transport {
            BedrockTransport::NetherNet(session) => session.client_public_key(),
            BedrockTransport::RakNet(_) => None,
        }
    }

    fn raknet_socket(&self) -> Option<&UdpSocket> {
        match &self.transport {
            BedrockTransport::RakNet(socket) => Some(socket),
            BedrockTransport::NetherNet(_) => None,
        }
    }

    /// Waits for already queued packets to reach the client.
    ///
    /// The `RakNet` transport writes synchronously, so the queueing paths that need this have
    /// already waited for the write before they return.
    pub async fn flush_reliable(&self, timeout: std::time::Duration) -> Result<(), String> {
        match &self.transport {
            BedrockTransport::NetherNet(session) => session.flush_reliable(timeout).await,
            BedrockTransport::RakNet(_) => Ok(()),
        }
    }

    /// Sends a raw `RakNet` datagram. Does nothing for transports that do their own framing.
    async fn send_datagram(&self, data: &[u8]) -> Result<(), Error> {
        let Some(socket) = self.raknet_socket() else {
            return Ok(());
        };
        socket.send_to(data, self.address).await.map(|_| ())
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

    /// Enables Minecraft's packet encryption for both directions, once the client has been sent
    /// the handshake that lets it derive the same key.
    pub async fn enable_encryption(
        &self,
        key: &[u8; 32],
    ) -> Result<(), EncryptionAlreadyEnabledError> {
        self.network_reader.lock().await.set_encryption(key)?;
        self.network_writer.write().await.set_encryption(key)
    }

    pub fn try_kick(&self, reason: DisconnectReason, message: String) {
        warn!("Closing connection for {}: {message}", self.address);
        let packet = CDisconnect::new(reason as i32, message);
        if let Ok(data) = self.serialize_packet(&packet) {
            let packet_len = data.len();
            let _ = self.pending_bytes.fetch_add(packet_len, Ordering::AcqRel);
            let _ = self
                .outgoing_packet_priority_send
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
        }
        self.close().await;
    }

    pub async fn send_chunks(&self, chunks: &[SyncChunk]) {
        let player = self.player.load_full();
        let Some(player) = player.as_ref() else {
            debug!(
                "send_chunks: player not set yet, dropping {} chunks",
                chunks.len()
            );
            return;
        };
        let Some(server) = player.world().server.upgrade() else {
            return;
        };

        let mut valid_chunks = Vec::with_capacity(chunks.len());
        for chunk in chunks {
            let mut event = ChunkSend::new(player.world(), chunk.clone());
            server.plugin_manager.fire(&server, &mut event).await;
            if !event.cancelled {
                valid_chunks.push(chunk.clone());
            }
        }

        if valid_chunks.is_empty() {
            return;
        }

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
        rayon::spawn(move || {
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
                        encoded_payloads.push(payload);
                        new_blobs.extend(blobs);
                    }
                    Err(e) => error!("Failed to serialize Bedrock chunk: {:?}", e),
                }
            }
            let _ = tx.send((encoded_payloads, new_blobs));
        });

        let Ok((encoded_payloads, new_blobs)) = rx.await else {
            return;
        };

        if !new_blobs.is_empty() {
            let mut cache = self
                .blob_cache
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for (hash, payload) in new_blobs {
                cache.insert(hash, payload);
            }
        }

        let mut packets_to_enqueue = Vec::with_capacity(encoded_payloads.len());
        {
            let encoder = self.network_writer.read().await;
            for payload in encoded_payloads {
                let mut packet_buf = Vec::new();
                match encoder.write_game_packet(
                    CLevelChunk::PACKET_ID as u16,
                    SubClient::Main,
                    SubClient::Main,
                    &payload,
                    &mut packet_buf,
                ) {
                    Ok(()) => packets_to_enqueue.push(packet_buf),
                    Err(err) => error!("Failed to write game packet wrapper: {err}"),
                }
            }
        }
        for packet_buf in packets_to_enqueue {
            self.enqueue_packet_data(packet_buf.into()).await;
        }
    }

    pub fn set_player(&self, player: Arc<Player>) {
        self.player.store(Arc::new(Some(player)));
    }

    #[allow(clippy::unused_async)]
    pub async fn enqueue_packet(&self, packet_data: Bytes) {
        self.try_enqueue_packet_data(packet_data);
    }

    pub fn try_enqueue_packet(&self, packet_data: Bytes) {
        self.try_enqueue_packet_data(packet_data);
    }

    /// Queues a clientbound packet to be sent to the connected client. Queued chunks are sent
    /// in-order to the client
    ///
    /// # Arguments
    ///
    /// * `packet_data`: A `Bytes` payload representing the encoded packet.
    #[allow(clippy::unused_async)]
    pub async fn enqueue_packet_data(&self, packet_data: Bytes) {
        self.try_enqueue_packet_data(packet_data);
    }

    pub fn try_enqueue_packet_data(&self, packet_data: Bytes) {
        if self.is_closed() {
            return;
        }

        let packet_len = packet_data.len();
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
            return;
        }

        if let Err(err) = self
            .outgoing_packet_queue_send
            .send(OutgoingPacket::normal(packet_data))
        {
            decrement_pending_bytes(&self.pending_bytes, packet_len);
            // This is expected to fail if we are closed
            if !self.is_closed() {
                error!("Failed to add packet to the outgoing packet queue for client: {err}");
            }
        }
    }

    pub fn write_raw_packet<P: BClientPacket>(
        packet: &P,
        mut writer: impl Write,
    ) -> Result<(), Error> {
        writer.write_all(&[P::PACKET_ID as u8])?;
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
        self.network_writer
            .try_read()
            .map_err(|_| Error::other("Bedrock packet encoder is busy"))?
            .serialize_packet(packet)
    }

    pub async fn send_packet<P: BClientPacket>(&self, packet: &P) {
        let mut data = Vec::new();
        match self.write_game_packet(packet, &mut data).await {
            Ok(()) => self.send_game_packet(data.into()).await,
            Err(err) => error!("Failed to serialize Bedrock packet: {err}"),
        }
    }

    pub async fn enqueue_client_packet<P: BClientPacket>(&self, packet: &P) {
        let mut data = Vec::new();
        match self.write_game_packet(packet, &mut data).await {
            Ok(()) => self.enqueue_packet(data.into()).await,
            Err(err) => error!("Failed to serialize Bedrock packet: {err}"),
        }
    }

    pub fn try_enqueue_client_packet<P: BClientPacket>(&self, packet: &P) {
        match self.serialize_packet(packet) {
            Ok(data) => self.try_enqueue_packet(data),
            Err(err) => error!("Failed to serialize Bedrock packet: {err}"),
        }
    }

    pub async fn send_game_packet(&self, packet_data: Bytes) {
        if self.is_closed() {
            return;
        }

        let packet_len = packet_data.len();
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
            return;
        }

        let (tx, rx) = oneshot::channel();
        if let Err(err) = self
            .outgoing_packet_priority_send
            .send(OutgoingPacket::priority(packet_data, tx))
        {
            decrement_pending_bytes(&self.pending_bytes, packet_len);
            if !self.is_closed() {
                error!("Failed to add priority packet to the outgoing packet queue: {err}");
            }
        } else {
            let _ = rx.await;
        }
    }

    pub async fn close(&self) {
        if self.close_token.is_cancelled() {
            return;
        }
        self.close_token.cancel();

        match &self.transport {
            BedrockTransport::RakNet(_) => {
                self.send_framed_packet(&CRakDisconnect, RakReliability::Unreliable)
                    .await;
            }
            BedrockTransport::NetherNet(session) => session.close().await,
        }

        self.be_clients.lock().await.remove(&self.address);
    }

    pub async fn await_tasks(&self) {
        self.tasks.close();
        self.tasks.wait().await;
    }

    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.close_token.is_cancelled()
            || matches!(&self.transport, BedrockTransport::NetherNet(session) if session.is_closed())
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

    pub async fn handle_login_sequence(
        self: &Arc<Self>,
        server: &Arc<Server>,
    ) -> PacketHandlerResult {
        while let Some(packet) = self.get_packet().await {
            let payload = &mut Cursor::new(&packet.payload);
            match packet.id {
                SRequestNetworkSettings::PACKET_ID => {
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
                }
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
                        Ok(Some(result)) => return result,
                        // RakNet must finish its encryption handshake first.
                        Ok(None) => {}
                        Err(err) => {
                            self.kick(DisconnectReason::Unknown, err.to_string()).await;
                            return PacketHandlerResult::Stop;
                        }
                    }
                }
                SClientToServerHandshake::PACKET_ID => {
                    let _packet = match SClientToServerHandshake::read(payload) {
                        Ok(p) => p,
                        Err(err) => {
                            error!("Failed to read SClientToServerHandshake: {err}");
                            continue;
                        }
                    };
                    let pending = self.pending_profile.swap(Arc::new(None));
                    if let Some(pending) = pending.as_ref() {
                        let (profile, new_config) = pending.as_ref();
                        self.send_login_success(server).await;
                        return PacketHandlerResult::ReadyToPlay(
                            profile.clone(),
                            new_config.clone(),
                        );
                    }
                    error!("Received ClientToServerHandshake but no pending profile was found.");
                    self.kick(DisconnectReason::BadPacket, "Handshake error".into())
                        .await;
                    return PacketHandlerResult::Stop;
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
                    client.handle_resource_pack_response(packet, &server_c).await;
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
            pumpkin_protocol::bedrock::server::item_stack_request::SItemStackRequest::PACKET_ID => {
                let packet = pumpkin_protocol::bedrock::server::item_stack_request::SItemStackRequest::read(reader)?;
                self.handle_item_stack_request(player, packet);
            }
            SInteract::PACKET_ID => {
                let packet = SInteract::read(reader)?;
                self.handle_interaction(&packet);
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
            SPlayerAction::PACKET_ID => {
                let packet = SPlayerAction::read(reader)?;
                self.handle_player_action(player, server, packet);
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
            pumpkin_protocol::bedrock::server::modal_form_response::SModalFormResponse::PACKET_ID => {
                let form_resp = pumpkin_protocol::bedrock::server::modal_form_response::SModalFormResponse::read(
                    reader,
                )?;
                self.handle_modal_form_response(player, server, form_resp);
            }
            SLoadingScreen::PACKET_ID => {
                // Ignore for now
            }
            SBlockPickRequest::PACKET_ID => {
                let packet = SBlockPickRequest::read(reader)?;
                self.handle_block_pick_request(player, &packet);
            }
            SRequestAbility::PACKET_ID => {
                self.handle_request_ability(player, &SRequestAbility::read(reader)?);
            }
            SMobEquipment::PACKET_ID => {
                let packet = SMobEquipment::read(reader)?;
                self.handle_mob_equipment(server, player, &packet);
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
        if packet.miss_hashes.is_empty() {
            return;
        }
        let missing_blobs = {
            let cache = self
                .blob_cache
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut missing_blobs = Vec::with_capacity(packet.miss_hashes.len());
            for hash in packet.miss_hashes {
                if let Some(payload) = cache.get(&hash) {
                    missing_blobs.push(MissingBlobData {
                        blob_id: hash,
                        blob_data: payload.clone(),
                    });
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
            let _guard = self.rt_handle.enter();
            Some(self.tasks.spawn(task))
        }
    }

    /// Writes `packet` as a single `RakNet` frame, without the Bedrock batch wrapper.
    pub async fn send_framed_packet<P: BClientPacket>(
        &self,
        packet: &P,
        reliability: RakReliability,
    ) {
        let mut packet_buf = Vec::new();
        match Self::write_raw_packet(packet, &mut packet_buf) {
            Ok(()) => self.send_framed_packet_data(packet_buf, reliability).await,
            Err(err) => error!("Failed to write framed packet: {err}"),
        }
    }

    /// Wraps an already encoded Bedrock batch (or internal `RakNet` message) into frames and sends
    /// them, splitting the payload when it does not fit a single datagram.
    pub async fn send_framed_packet_data(
        &self,
        packet_buf: Vec<u8>,
        mut reliability: RakReliability,
    ) {
        let mut split_size = 0;
        let mut split_id = 0;
        let mut order_index = 0;

        let mut max_content_len =
            MTU - UDP_HEADER_SIZE - 12 - if reliability.is_ordered() { 4 } else { 0 };

        let count = if packet_buf.len() > max_content_len {
            reliability = RakReliability::ReliableOrdered;
            split_id = self.output_split_number.fetch_add(1, Ordering::Relaxed);
            max_content_len = SPLIT_FRAME_MAX_CONTENT;
            split_size = packet_buf.len().div_ceil(max_content_len) as u32;
            split_size as usize
        } else {
            1
        };

        if reliability.is_ordered() {
            order_index = self.output_ordered_index.fetch_add(1, Ordering::Relaxed);
        }

        for i in 0..count {
            let end = if i + 1 == count && !packet_buf.len().is_multiple_of(max_content_len) {
                packet_buf.len() % max_content_len
            } else {
                max_content_len
            };
            let chunk = &packet_buf[i * max_content_len..i * max_content_len + end];

            let mut frame_set = FrameSet {
                sequence: u24(0),
                frames: Vec::with_capacity(1),
            };

            let mut frame = Frame {
                payload: chunk.to_vec(),
                reliability,
                split_index: i as u32,
                reliable_number: 0,
                sequence_index: 0,
                order_index,
                order_channel: 0,
                split_size,
                split_id,
            };

            if reliability.is_reliable() {
                frame.reliable_number = self.output_reliable_number.fetch_add(1, Ordering::Relaxed);
            }

            frame_set.frames.push(frame);

            // 0x84 marks the start of a split packet, 0x8c every following fragment.
            let id = if i == 0 { 0x84 } else { 0x8c };
            self.send_frame_set(frame_set, id).await;
        }
    }

    pub async fn send_frame_set(&self, mut frame_set: FrameSet, id: u8) {
        if self.raknet_socket().is_none() {
            return;
        }
        let sequence = self.output_sequence_number.fetch_add(1, Ordering::Relaxed);
        frame_set.sequence = u24(sequence);

        let mut frame_set_buf = Vec::new();
        if let Err(err) = frame_set.write_packet_data(&mut frame_set_buf, id) {
            error!("Failed to write frame set data: {err}");
            return;
        }

        if frame_set.frames.iter().any(|f| f.reliability.is_reliable()) {
            self.unacked_outgoing_frames.lock().await.insert(
                sequence,
                (id, frame_set_buf.clone(), std::time::Instant::now()),
            );
        }

        if let Err(err) = self.send_datagram(&frame_set_buf).await
            && !self.is_closed()
        {
            warn!("Failed to send packet to client {}: {}", self.address, err);
            self.close_token.cancel();
        }
    }

    pub async fn send_acknowledgement(&self, ack: &Acknowledge, id: u8) -> Result<(), Error> {
        if self.raknet_socket().is_none() {
            return Ok(());
        }
        let mut packet_buf = Vec::new();
        ack.write(&mut packet_buf, id)?;

        if let Err(err) = self.send_datagram(&packet_buf).await {
            warn!("Failed to send acknowledgement to {}: {err}", self.address);
            self.close().await;
            return Err(err);
        }
        Ok(())
    }

    /// Dispatches one datagram received on the client's `RakNet` socket.
    pub async fn handle_packet_payload(
        self: &Arc<Self>,
        server: &Arc<Server>,
        packet: Bytes,
    ) -> Result<(), Error> {
        let reader = &mut Cursor::new(packet);

        match u8::read(reader)? {
            RAKNET_ACK => {
                self.handle_ack(&Acknowledge::read(reader)?).await;
            }
            RAKNET_NACK => {
                self.handle_nack(&Acknowledge::read(reader)?).await;
            }
            RAKNET_VALID..=0x8d => {
                self.handle_frame_set(server, FrameSet::read(reader)?)
                    .await?;
            }
            id => {
                warn!("Bedrock: Received unknown packet header {id}");
            }
        }
        Ok(())
    }

    async fn handle_ack(&self, ack: &Acknowledge) {
        let mut unacked = self.unacked_outgoing_frames.lock().await;
        for seq in &ack.sequences {
            unacked.remove(seq);
        }
    }

    async fn handle_nack(&self, nack: &Acknowledge) {
        debug!("Received NACK for sequences: {:?}", nack.sequences);
        let mut resend_data = Vec::new();
        {
            let unacked = self.unacked_outgoing_frames.lock().await;
            for seq in &nack.sequences {
                if let Some((_id, data, _timestamp)) = unacked.get(seq) {
                    resend_data.push(data.clone());
                }
            }
        }

        for data in resend_data {
            if let Err(err) = self.send_datagram(&data).await {
                warn!("Failed to resend packet from NACK: {err}");
            }
        }
    }

    async fn handle_frame_set(
        self: &Arc<Self>,
        server: &Arc<Server>,
        frame_set: FrameSet,
    ) -> Result<(), Error> {
        let sequence = frame_set.sequence.0;

        {
            let mut received = self.received_sequences.lock().await;
            if received.contains(&sequence) {
                debug!("Received duplicate RakNet sequence: {sequence}");
                return Ok(());
            }
            received.insert(sequence);
            // Limit the size of received sequences to avoid memory leak
            if received.len() > 4096 {
                received.clear();
            }
        }

        self.pending_acks.lock().await.push(sequence);

        for frame in frame_set.frames {
            self.handle_frame(server, frame).await?;
        }
        Ok(())
    }

    async fn handle_frame(
        self: &Arc<Self>,
        server: &Arc<Server>,
        mut frame: Frame,
    ) -> Result<(), Error> {
        if frame.split_size > 0 {
            if frame.split_size > MAX_INBOUND_SPLIT_SIZE {
                return Err(Error::other("RakNet split frame count exceeds the limit"));
            }
            let fragment_index = frame.split_index as usize;
            let compound_id = frame.split_id;
            let mut compounds = self.compounds.lock().await;

            let entry = compounds.entry(compound_id).or_insert_with(|| {
                let mut vec = Vec::with_capacity(frame.split_size as usize);
                vec.resize_with(frame.split_size as usize, || None);
                vec
            });

            if fragment_index >= entry.len() {
                return Err(Error::other(format!(
                    "Fragment index {fragment_index} out of bounds for size {}",
                    entry.len()
                )));
            }

            entry[fragment_index] = Some(frame);

            if entry.iter().any(Option::is_none) {
                return Ok(());
            }

            let mut frames_opt = compounds
                .remove(&compound_id)
                .ok_or_else(|| Error::other("Compound ID vanished"))?;

            let total_len: usize = frames_opt.iter().flatten().map(|f| f.payload.len()).sum();
            let mut merged = Vec::with_capacity(total_len);
            for f in frames_opt.iter().flatten() {
                merged.extend_from_slice(&f.payload);
            }

            frame = frames_opt[0]
                .take()
                .ok_or_else(|| Error::other("Failed to retrieve primary frame"))?;

            frame.payload = merged;
            frame.split_size = 0;
        }

        // Handling Sequencing
        if frame.reliability.is_sequenced() {
            let mut highest_sequenced = self.highest_sequence_index.lock().await;
            let current_highest = highest_sequenced.entry(frame.order_channel).or_insert(0);
            if frame.sequence_index < *current_highest {
                return Ok(());
            }
            *current_highest = frame.sequence_index;
        }

        // Handling Ordering
        if frame.reliability.is_ordered() {
            let mut expected_order = self.expected_order_index.lock().await;
            let expected = expected_order.entry(frame.order_channel).or_insert(0);

            if frame.order_index == *expected {
                *expected += 1;
                self.process_frame_payload(server, frame.payload).await?;

                // Check for queued frames
                let mut ordered_queues = self.ordered_queues.lock().await;
                if let Some(queue) = ordered_queues.get_mut(&frame.order_channel) {
                    while let Some(next_frame) = queue.remove(expected) {
                        *expected += 1;
                        self.process_frame_payload(server, next_frame.payload)
                            .await?;
                    }
                }
            } else if frame.order_index > *expected {
                let mut ordered_queues = self.ordered_queues.lock().await;
                let queue = ordered_queues
                    .entry(frame.order_channel)
                    .or_insert_with(BTreeMap::new);
                if queue.len() >= MAX_ORDERED_QUEUE_SIZE && !queue.contains_key(&frame.order_index)
                {
                    return Err(Error::other("RakNet ordered frame queue is full"));
                }
                queue.insert(frame.order_index, frame);
            }
            // If frame.order_index < *expected, it's an old frame, discard it.
        } else {
            self.process_frame_payload(server, frame.payload).await?;
        }

        Ok(())
    }

    async fn process_frame_payload(
        self: &Arc<Self>,
        server: &Arc<Server>,
        payload: Vec<u8>,
    ) -> Result<(), Error> {
        if payload.is_empty() {
            return Ok(());
        }
        let id = payload[0];

        if id == BEDROCK_GAME_PACKET {
            self.process_batch(server, payload).await
        } else {
            // An internal RakNet message, such as a connected ping.
            let mut cursor = Cursor::new(payload);
            let _id = u8::read(&mut cursor)?;
            self.handle_raknet_packet(i32::from(id), cursor).await
        }
    }

    async fn handle_raknet_packet(
        self: &Arc<Self>,
        packet_id: i32,
        mut payload: Cursor<Vec<u8>>,
    ) -> Result<(), Error> {
        let reader = &mut payload;
        match packet_id {
            SConnectionRequest::PACKET_ID => {
                let request = SConnectionRequest::read(reader)?;

                self.send_framed_packet(
                    &CConnectionRequestAccepted::new(
                        self.address,
                        0,
                        [SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 19132)); 10],
                        request.time,
                        UNIX_EPOCH.elapsed().unwrap_or_default().as_millis() as u64,
                    ),
                    RakReliability::Unreliable,
                )
                .await;
            }
            SNewIncomingConnection::PACKET_ID => {
                SNewIncomingConnection::read(reader)?;
            }
            SConnectedPing::PACKET_ID => {
                self.handle_connected_ping(SConnectedPing::read(reader)?)
                    .await;
            }
            SDisconnect::PACKET_ID | SConnectionLost::PACKET_ID => {
                self.close().await;
            }
            _ => {
                warn!("Bedrock: Received Unknown RakNet Online packet: {packet_id}");
            }
        }
        Ok(())
    }

    /// Handles the `RakNet` packets a client sends before a session exists.
    #[expect(clippy::too_many_lines)]
    pub async fn handle_offline_packet(
        server: &Server,
        packet_id: u8,
        payload: &mut Cursor<&[u8]>,
        addr: SocketAddr,
        socket: &UdpSocket,
        be_clients: &Arc<Mutex<HashMap<SocketAddr, Arc<Self>>>>,
    ) -> Result<(), Error> {
        let packet_id_i32 = i32::from(packet_id);
        if packet_id_i32 == SOpenConnectionRequest1::PACKET_ID {
            let is_banned = {
                let mut banned_ips = server
                    .data
                    .banned_ip_list
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                banned_ips.get_entry(&addr.ip()).is_some()
            };
            if is_banned {
                Self::send_offline_packet(
                    &CConnectionBanned::new(server.server_guid),
                    addr,
                    socket,
                )
                .await;
                return Ok(());
            }

            let player_count = {
                let status = server
                    .get_status()
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                status
                    .status_response
                    .players
                    .as_ref()
                    .map_or(0, |p| p.online)
            };
            if player_count >= server.advanced_config.networking.bedrock.max_players {
                Self::send_offline_packet(
                    &CNoFreeIncomingConnections::new(server.server_guid),
                    addr,
                    socket,
                )
                .await;
                return Ok(());
            }

            let old_client = {
                let mut clients_guard = be_clients.lock().await;
                clients_guard.remove(&addr)
            };
            if let Some(client) = old_client {
                debug!(
                    "Closing old Bedrock client connection for {} due to new connection request",
                    addr
                );
                client.close().await;
            }
        }

        match packet_id_i32 {
            SUnconnectedPing::PACKET_ID | SUnconnectedPingOpenConnections::PACKET_ID
                if !advertises_raknet(
                    server.advanced_config.networking.bedrock.raknet_status,
                    addr.ip(),
                ) => {}
            SUnconnectedPing::PACKET_ID => {
                Self::handle_unconnected_ping(
                    server,
                    SUnconnectedPing::read(payload)?,
                    addr,
                    socket,
                )
                .await;
            }
            SUnconnectedPingOpenConnections::PACKET_ID => {
                let packet = SUnconnectedPingOpenConnections::read(payload)?;
                Self::handle_unconnected_ping(
                    server,
                    SUnconnectedPing {
                        time: packet.time,
                        magic: packet.magic,
                        client_guid: packet.client_guid,
                    },
                    addr,
                    socket,
                )
                .await;
            }
            SOpenConnectionRequest1::PACKET_ID => {
                Self::handle_open_connection_1(
                    server,
                    SOpenConnectionRequest1::read(payload)?,
                    addr,
                    socket,
                )
                .await;
            }
            SOpenConnectionRequest2::PACKET_ID => {
                let is_already_connected = {
                    let clients_guard = be_clients.lock().await;
                    clients_guard.contains_key(&addr)
                };
                if is_already_connected {
                    Self::send_offline_packet(
                        &CAlreadyConnected::new(server.server_guid),
                        addr,
                        socket,
                    )
                    .await;
                    return Ok(());
                }

                Self::handle_open_connection_2(
                    server,
                    SOpenConnectionRequest2::read(payload)?,
                    addr,
                    socket,
                )
                .await;
            }
            _ => error!("Bedrock: Received Unknown RakNet Offline packet: {packet_id}"),
        }
        Ok(())
    }

    /// Writes a packet that is not part of a `RakNet` session, such as a status pong.
    pub async fn send_offline_packet<P: BClientPacket>(
        packet: &P,
        addr: SocketAddr,
        socket: &UdpSocket,
    ) {
        let mut data = Vec::new();
        if let Err(err) = Self::write_raw_packet(packet, &mut data) {
            error!("Failed to write offline packet: {err}");
            return;
        }
        // We dont care if it works, if not the client will try again!
        let _ = socket.send_to(&data, addr).await;
    }
}
