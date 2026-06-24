pub mod play;
use crossbeam::atomic::AtomicCell;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    io::{Cursor, Error, Write},
    net::{Ipv4Addr, SocketAddrV4},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU16, AtomicU32, Ordering},
    },
    time::UNIX_EPOCH,
};

use tracing::{debug, error, warn};

use bytes::Bytes;
use pumpkin_config::networking::compression::CompressionInfo;
use pumpkin_protocol::{
    BClientPacket, PacketDecodeError, RawPacket,
    bedrock::{
        MTU, RAKNET_ACK, RAKNET_GAME_PACKET, RAKNET_NACK, RakReliability, SPLIT_FRAME_MAX_CONTENT,
        SubClient, UDP_HEADER_SIZE,
        ack::Acknowledge,
        client::{
            disconnect_player::CDisconnectPlayer, level_chunk::CLevelChunk,
            raknet::connection::CConnectionRequestAccepted,
        },
        frame_set::{Frame, FrameSet},
        packet_decoder::UDPNetworkDecoder,
        packet_encoder::UDPNetworkEncoder,
        server::{
            animate::SAnimate,
            block_pick_request::SBlockPickRequest,
            client_cache_status::SClientCacheStatus,
            command_request::SCommandRequest,
            container_close::SContainerClose,
            emote::SEmote,
            interaction::SInteraction,
            inventory_transaction::SInventoryTransaction,
            loading_screen::SLoadingScreen,
            login::SLogin,
            mob_equipment::SMobEquipment,
            player_action::SPlayerAction,
            player_auth_input::SPlayerAuthInput,
            raknet::{
                connection::{
                    SConnectedPing, SConnectionRequest, SDisconnect, SNewIncomingConnection,
                },
                open_connection::{SOpenConnectionRequest1, SOpenConnectionRequest2},
                unconnected_ping::SUnconnectedPing,
            },
            request_chunk_radius::SRequestChunkRadius,
            request_network_settings::SRequestNetworkSettings,
            resource_pack_response::SResourcePackResponse,
            set_local_player_as_initialized::SSetLocalPlayerAsInitialized,
            text::SText,
        },
    },
    codec::u24,
    packet::Packet,
    serial::PacketRead,
};
use std::net::SocketAddr;
use tokio::{
    net::UdpSocket,
    sync::mpsc::{Receiver, Sender},
    sync::{Mutex, RwLock, oneshot},
    task::JoinHandle,
};

use tokio_util::{sync::CancellationToken, task::TaskTracker};

pub mod connection;
pub mod login;
pub mod open_connection;
pub mod unconnected;
use crate::{
    entity::player::Player,
    net::{DisconnectReason, PacketHandlerResult},
    plugin::api::events::world::chunk_send::ChunkSend,
    server::Server,
};
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

pub struct BedrockClient {
    socket: Arc<UdpSocket>,
    /// The client's IP address.
    pub address: SocketAddr,
    pub player: Mutex<Option<Arc<Player>>>,
    pub version: AtomicCell<BedrockMinecraftVersion>,
    pub client_data: ArcSwap<Option<Arc<ClientData>>>,
    /// All Bedrock clients
    /// This list is used to remove the client if the connection gets closed
    pub be_clients: Arc<Mutex<HashMap<SocketAddr, Arc<Self>>>>,

    tasks: TaskTracker,
    outgoing_packet_queue_send: Sender<OutgoingPacket>,
    /// A queue of serialized packets to send to the network
    outgoing_packet_queue_recv: Mutex<Option<Receiver<OutgoingPacket>>>,

    outgoing_packet_priority_send: Sender<OutgoingPacket>,
    outgoing_packet_priority_recv: Mutex<Option<Receiver<OutgoingPacket>>>,

    /// The packet encoder for outgoing packets.
    network_writer: Arc<RwLock<UDPNetworkEncoder>>,
    /// The packet decoder for incoming packets.
    network_reader: Mutex<UDPNetworkDecoder>,

    _use_frame_sets: AtomicBool,
    output_sequence_number: AtomicU32,
    output_reliable_number: AtomicU32,
    output_split_number: AtomicU16,
    output_sequenced_index: AtomicU32,
    output_ordered_index: AtomicU32,
    /// The next form ID to use for custom forms.
    pub next_form_id: AtomicU32,
    pub inventory_opened: AtomicBool,
    /// An notifier that is triggered when this client is closed.
    close_token: CancellationToken,
    last_seen: Arc<AtomicCell<std::time::Instant>>,
    /// Store Fragments until the packet is complete
    compounds: Arc<Mutex<HashMap<u16, Vec<Option<Frame>>>>>,
    //input_sequence_number: AtomicU32,
    received_sequences: Mutex<HashSet<u32>>,
    pending_acks: Mutex<Vec<u32>>,
    #[allow(clippy::type_complexity)]
    unacked_outgoing_frames: Mutex<HashMap<u32, (u8, Vec<u8>, std::time::Instant)>>,
    expected_order_index: Mutex<HashMap<u8, u32>>,
    highest_sequence_index: Mutex<HashMap<u8, u32>>,
    ordered_queues: Mutex<HashMap<u8, BTreeMap<u32, Frame>>>,
    incoming_game_packet_send: Sender<RawPacket>,
    incoming_game_packet_recv: Mutex<Option<Receiver<RawPacket>>>,
}

impl BedrockClient {
    #[must_use]
    pub fn new(
        socket: Arc<UdpSocket>,
        address: SocketAddr,
        be_clients: Arc<Mutex<HashMap<SocketAddr, Arc<Self>>>>,
    ) -> Self {
        let (send, recv) = tokio::sync::mpsc::channel(4096);
        let (priority_send, priority_recv) = tokio::sync::mpsc::channel(4096);
        let (incoming_send, incoming_recv) = tokio::sync::mpsc::channel(4096);
        Self {
            socket,
            player: Mutex::new(None),
            address,
            version: AtomicCell::new(BedrockMinecraftVersion::Unknown),
            client_data: ArcSwap::new(Arc::new(None)),
            be_clients,
            network_writer: Arc::new(RwLock::new(UDPNetworkEncoder::new())),
            network_reader: Mutex::new(UDPNetworkDecoder::new()),
            tasks: TaskTracker::new(),
            outgoing_packet_queue_send: send,
            outgoing_packet_queue_recv: Mutex::new(Some(recv)),
            outgoing_packet_priority_send: priority_send,
            outgoing_packet_priority_recv: Mutex::new(Some(priority_recv)),
            _use_frame_sets: AtomicBool::new(false),
            output_sequence_number: AtomicU32::new(0),
            output_reliable_number: AtomicU32::new(0),
            output_split_number: AtomicU16::new(0),
            output_sequenced_index: AtomicU32::new(0),
            output_ordered_index: AtomicU32::new(0),
            next_form_id: AtomicU32::new(0),
            inventory_opened: AtomicBool::new(false),
            compounds: Arc::new(Mutex::new(HashMap::new())),
            close_token: CancellationToken::new(),
            last_seen: Arc::new(AtomicCell::new(std::time::Instant::now())),
            received_sequences: Mutex::new(HashSet::new()),
            pending_acks: Mutex::new(Vec::new()),
            unacked_outgoing_frames: Mutex::new(HashMap::new()),
            expected_order_index: Mutex::new(HashMap::new()),
            highest_sequence_index: Mutex::new(HashMap::new()),
            ordered_queues: Mutex::new(HashMap::new()),
            //input_sequence_number: AtomicU32::new(0),
            incoming_game_packet_send: incoming_send,
            incoming_game_packet_recv: Mutex::new(Some(incoming_recv)),
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
        let client = self.clone();
        self.spawn_task(async move {
            let mut packet_receiver = {
                let mut guard = client.outgoing_packet_queue_recv.lock().await;
                guard
                    .take()
                    .expect("Outgoing packet receiver was already taken")
            };
            let mut priority_packet_receiver = {
                let mut guard = client.outgoing_packet_priority_recv.lock().await;
                guard
                    .take()
                    .expect("Outgoing packet receiver was already taken")
            };
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(100));

            while !client.close_token.is_cancelled() {
                let packet = tokio::select! {
                    biased;
                    () = client.close_token.cancelled() => break,
                    res = priority_packet_receiver.recv() => match res {
                        Some(p) => p,
                        None => break,
                    },
                    _ = interval.tick() => {
                        // Check for timeout (10 seconds)
                        if client.last_seen.load().elapsed() > std::time::Duration::from_secs(10) {
                            debug!("Bedrock client {} timed out", client.address);
                            client.close().await;
                            break;
                        }

                        // Flush ACKs
                        let mut pending = client.pending_acks.lock().await;
                        if !pending.is_empty() {
                            let ack = Acknowledge::new(pending.clone());
                            pending.clear();
                            let _ = client.send_acknowledgement(&ack, RAKNET_ACK).await;
                        }

                        // Check retransmission
                        let now = std::time::Instant::now();
                        let mut resend = Vec::new();
                        {
                            let mut unacked = client.unacked_outgoing_frames.lock().await;
                            for (seq, (id, data, timestamp)) in unacked.iter_mut() {
                                if now.duration_since(*timestamp) > std::time::Duration::from_secs(1) {
                                    resend.push((*seq, *id, data.clone()));
                                    // Update timestamp
                                    *timestamp = now;
                                    // Limit resends per tick to avoid starvation
                                    if resend.len() >= 50 {
                                        break;
                                    }
                                }
                            }
                        }

                        if !resend.is_empty() {
                            let encoder = client.network_writer.read().await;
                            for (seq, id, data) in resend {
                                debug!("Resending reliable sequence {} (ID: {})", seq, id);
                                if let Err(err) = encoder.write_packet(&data, client.address, &client.socket).await {
                                    warn!("Failed to resend packet for sequence {}: {}", seq, err);
                                }
                            }
                        }
                        continue;
                    }
                    res = packet_receiver.recv() => match res {
                        Some(p) => p,
                        None => break,
                    },
                };

                client
                    .send_framed_packet_data(packet.data.to_vec(), RakReliability::ReliableOrdered)
                    .await;

                if let Some(completion) = packet.completion {
                    let _ = completion.send(());
                }
            }
        });
    }

    pub async fn process_packet(self: &Arc<Self>, server: &Arc<Server>, packet: Bytes) {
        self.last_seen.store(std::time::Instant::now());
        if let Err(error) = self.handle_packet_payload(server, packet).await {
            error!(
                "Failed to handle packet payload for {}: {}",
                self.address, error
            );
            self.kick(DisconnectReason::BadPacket, error.to_string())
                .await;
        }
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

    pub async fn kick(&self, reason: DisconnectReason, message: String) {
        self.send_game_packet(&CDisconnectPlayer::new(reason as i32, message))
            .await;
        self.close().await;
    }

    pub async fn send_chunks(&self, chunks: &[SyncChunk]) {
        let player = self.player.lock().await.clone();
        let Some(player) = player.as_ref() else {
            return;
        };
        let Some(server) = player.world().server.upgrade() else {
            return;
        };

        for chunk in chunks {
            let event = ChunkSend::new(player.world(), chunk.clone());
            let event = server.plugin_manager.fire(event).await;
            if event.cancelled {
                continue;
            }

            self.enqueue_packet_internal(&CLevelChunk {
                dimension: 0,
                cache_enabled: false,
                chunk,
            })
            .await;
        }
    }

    pub async fn enqueue_packet<P: BClientPacket>(&self, packet: &P) {
        let mut packet_buf = Vec::new();
        match self.write_game_packet(packet, &mut packet_buf).await {
            Ok(()) => {
                let payload = Bytes::from(packet_buf);
                let player = self.player.lock().await.clone();
                let cancelled = if let Some(player) = player.as_ref() {
                    player
                        .fire_packet_sent_no_obj(P::PACKET_ID, payload.clone())
                        .await
                } else {
                    false
                };
                if !cancelled {
                    self.enqueue_packet_data(payload).await;
                }
            }
            Err(err) => error!("Failed to write game packet: {err}"),
        }
    }

    pub async fn enqueue_packet_internal<P: BClientPacket>(&self, packet: &P) {
        let mut packet_buf = Vec::new();
        match self.write_game_packet(packet, &mut packet_buf).await {
            Ok(()) => self.enqueue_packet_data(packet_buf.into()).await,
            Err(err) => error!("Failed to write game packet: {err}"),
        }
    }

    pub fn try_enqueue_packet<P: BClientPacket>(&self, packet: &P) {
        let mut packet_buf = Vec::new();
        let mut packet_payload = Vec::new();
        if let Err(err) = packet.write_packet(&mut packet_payload) {
            error!("Failed to write packet for try_enqueue_packet: {err}");
            return;
        }

        {
            let Ok(network_writer) = self.network_writer.try_read() else {
                debug!("Failed to lock network writer for try_enqueue_packet");
                return;
            };

            if let Err(err) = network_writer.write_game_packet(
                P::PACKET_ID as u16,
                SubClient::Main,
                SubClient::Main,
                &packet_payload,
                &mut packet_buf,
            ) {
                error!("Failed to write game packet for try_enqueue_packet: {err}");
                return;
            }
        }

        self.try_enqueue_packet_data(packet_buf.into());
    }

    /// Queues a clientbound packet to be sent to the connected client. Queued chunks are sent
    /// in-order to the client
    ///
    /// # Arguments
    ///
    /// * `packet`: A reference to a packet object implementing the `ClientPacket` trait.
    pub async fn enqueue_packet_data(&self, packet_data: Bytes) {
        if let Err(err) = self
            .outgoing_packet_queue_send
            .send(OutgoingPacket::normal(packet_data))
            .await
        {
            // This is expected to fail if we are closed
            if !self.is_closed() {
                error!("Failed to add packet to the outgoing packet queue for client: {err}");
            }
        }
    }

    pub fn try_enqueue_packet_data(&self, packet_data: Bytes) {
        if let Err(err) = self
            .outgoing_packet_queue_send
            .try_send(OutgoingPacket::normal(packet_data))
        {
            match err {
                tokio::sync::mpsc::error::TrySendError::Full(_) => {
                    debug!(
                        "Failed to add packet to the outgoing packet queue for client: channel full"
                    );
                }
                tokio::sync::mpsc::error::TrySendError::Closed(_) => {
                    if !self.is_closed() {
                        error!(
                            "Failed to add packet to the outgoing packet queue for client: channel closed"
                        );
                    }
                }
            }
        }
    }

    pub fn write_raw_packet<P: BClientPacket>(
        packet: &P,
        mut write: impl Write,
    ) -> Result<(), Error> {
        write.write_all(&[P::PACKET_ID as u8])?;
        packet.write_packet(write)
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

    pub async fn send_game_packet<P: BClientPacket>(&self, packet: &P) {
        let mut packet_buf = Vec::new();
        match self.write_game_packet(packet, &mut packet_buf).await {
            Ok(()) => {
                let payload = Bytes::from(packet_buf);
                let player = self.player.lock().await.clone();
                let cancelled = if let Some(player) = player.as_ref() {
                    player
                        .fire_packet_sent_no_obj(P::PACKET_ID, payload.clone())
                        .await
                } else {
                    false
                };
                if cancelled {
                    return;
                }
                let (tx, rx) = oneshot::channel();
                if let Err(err) = self
                    .outgoing_packet_priority_send
                    .send(OutgoingPacket::priority(payload, tx))
                    .await
                {
                    if !self.is_closed() {
                        error!("Failed to add priority packet to the outgoing packet queue: {err}");
                    }
                } else {
                    let _ = rx.await;
                }
            }
            Err(err) => error!("Failed to write game packet: {err}"),
        }
    }

    pub async fn write_game_packet_to_set<P: BClientPacket>(
        &self,
        packet: &P,
        frame_set: &mut FrameSet,
    ) {
        let mut payload = Vec::new();
        match self.write_game_packet(packet, &mut payload).await {
            Ok(()) => {
                frame_set.frames.push(Frame::new_unreliable(payload));
            }
            Err(err) => error!("Failed to write game packet to set: {err}"),
        }
    }

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

            if reliability.is_sequenced() {
                frame.sequence_index = self.output_sequenced_index.fetch_add(1, Ordering::Relaxed);
            }

            frame_set.frames.push(frame);

            let id = if i == 0 { 0x84 } else { 0x8c };
            self.send_frame_set(frame_set, id).await;
        }
    }

    pub async fn send_frame_set(&self, mut frame_set: FrameSet, id: u8) {
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

        if let Err(err) = self
            .network_writer
            .read()
            .await
            .write_packet(&frame_set_buf, self.address, &self.socket)
            .await
            && !self.is_closed()
        {
            warn!("Failed to send packet to client {}: {}", self.address, err);
            self.close_token.cancel();
        }
    }

    pub async fn close(&self) {
        if self.is_closed() {
            return;
        }
        self.close_token.cancel();
        self.be_clients.lock().await.remove(&self.address);
    }

    pub async fn await_tasks(&self) {
        self.tasks.close();
        self.tasks.wait().await;
    }

    pub fn is_closed(&self) -> bool {
        self.close_token.is_cancelled()
    }

    pub fn enqueue_spawn_packet(self: &Arc<Self>, entity: Arc<dyn crate::entity::EntityBase>) {
        let client = self.clone();
        self.spawn_task(async move {
            entity.send_bedrock_spawn_packet(&client).await;
        });
    }

    pub async fn send_acknowledgement(&self, ack: &Acknowledge, id: u8) -> Result<(), Error> {
        let mut packet_buf = Vec::new();
        ack.write(&mut packet_buf, id)?;

        if let Err(err) = self
            .network_writer
            .read()
            .await
            .write_packet(&packet_buf, self.address, &self.socket)
            .await
        {
            warn!("Failed to send acknowledgement to {}: {err}", self.address);
            self.close().await;
            return Err(err);
        }
        Ok(())
    }

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
            0x80..=0x8d => {
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
            if let Err(err) = self
                .network_writer
                .read()
                .await
                .write_packet(&data, self.address, &self.socket)
                .await
            {
                warn!("Failed to resend packet from NACK: {}", err);
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
                debug!("Received duplicate RakNet sequence: {}", sequence);
                return Ok(());
            }
            received.insert(sequence);
            // Limit the size of received sequences to avoid memory leak
            if received.len() > 4096 {
                // This is a very simple way to clear it, ideally we'd use a sliding window
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

            // Check if all fragments are received
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

        if id == RAKNET_GAME_PACKET as u8 {
            // Decompress the batch
            let decompressed_payload = self
                .get_packet_payload(payload)
                .await
                .ok_or_else(|| Error::other("Failed to decompress game packet batch"))?;

            // Loop through the decompressed buffer to extract ALL batched packets
            let mut cursor = Cursor::new(decompressed_payload);

            while (cursor.position() as usize) < cursor.get_ref().len() {
                let game_packet = self
                    .network_reader
                    .lock()
                    .await
                    .get_game_packet(&mut cursor)
                    .map_err(|e| Error::other(e.to_string()))?;

                self.handle_game_packet(server, game_packet).await?;
            }
        } else {
            // It's an internal RakNet message (like SConnectedPing)
            let mut cursor = Cursor::new(payload);
            let _id = u8::read(&mut cursor)?; // consume ID byte
            self.handle_raknet_packet(i32::from(id), cursor).await?;
        }

        Ok(())
    }

    async fn handle_game_packet(
        &self,
        _server: &Arc<Server>,
        packet: RawPacket,
    ) -> Result<(), Error> {
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
                    self.handle_request_network_settings(packet, server).await;
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

    pub async fn progress_player_packets(
        self: &Arc<Self>,
        player: &Arc<Player>,
        server: &Arc<Server>,
    ) {
        while let Some(packet) = self.get_packet().await {
            let mut event = crate::plugin::server::packet::PacketReceivedEvent::new(
                player.clone(),
                packet.id,
                packet.payload.clone(),
            );
            event = server.plugin_manager.fire(event).await;
            if event.cancelled {
                continue;
            }

            if let Err(err) = self.handle_play_packet(player, server, packet).await {
                error!("Failed to handle Bedrock play packet: {err}");
            }
        }
    }

    pub async fn handle_play_packet(
        &self,
        player: &Arc<Player>,
        server: &Arc<Server>,
        packet: RawPacket,
    ) -> Result<(), Error> {
        let payload = &packet.payload[..];
        let reader = &mut &payload[..];
        match packet.id {
            SClientCacheStatus::PACKET_ID => {
                // TODO
            }
            SResourcePackResponse::PACKET_ID => {
                self.handle_resource_pack_response(SResourcePackResponse::read(reader)?, server)
                    .await;
            }
            SPlayerAuthInput::PACKET_ID => {
                self.handle_player_auth_input(player, SPlayerAuthInput::read(reader)?, server)
                    .await;
            }
            SRequestChunkRadius::PACKET_ID => {
                self.handle_request_chunk_radius(player, SRequestChunkRadius::read(reader)?)
                    .await;
            }
            SInventoryTransaction::PACKET_ID => {
                self.handle_inventory_action(player, SInventoryTransaction::read(reader)?).await;
            }
            pumpkin_protocol::bedrock::server::item_stack_request::SItemStackRequest::PACKET_ID => {
                self.handle_item_stack_request(player, pumpkin_protocol::bedrock::server::item_stack_request::SItemStackRequest::read(reader)?).await;
            }
            SInteraction::PACKET_ID => {
                self.handle_interaction(player, SInteraction::read(reader)?)
                    .await;
            }
            SContainerClose::PACKET_ID => {
                self.handle_container_close(player, SContainerClose::read(reader)?)
                    .await;
            }
            SText::PACKET_ID => {
                self.handle_chat_message(server, player, SText::read(reader)?)
                    .await;
            }
            SCommandRequest::PACKET_ID => {
                self.handle_chat_command(player, server, SCommandRequest::read(reader)?)
                    .await;
            }
            SSetLocalPlayerAsInitialized::PACKET_ID => {
                self.handle_set_local_player_as_initialized(
                    player,
                    &SSetLocalPlayerAsInitialized::read(reader)?,
                );
            }
            SPlayerAction::PACKET_ID => {
                self.handle_player_action(player, server, SPlayerAction::read(reader)?)
                    .await;
            }
            SAnimate::PACKET_ID => {
                self.handle_animate(player, server, &SAnimate::read(reader)?).await;
            }
            SEmote::PACKET_ID => {
                self.handle_emote(player, server, SEmote::read(reader)?).await;
            }
            // SEmoteList::PACKET_ID => {
            //     self.handle_emote_list(player, server, SEmoteList::read(reader)?);
            // }
            pumpkin_protocol::bedrock::server::modal_form_response::SModalFormResponse::PACKET_ID => {
                self.handle_modal_form_response(
                    player,
                    server,
                    pumpkin_protocol::bedrock::server::modal_form_response::SModalFormResponse::read(
                        reader,
                    )?,
                )
                .await;
            }
            SLoadingScreen::PACKET_ID => {
                // Ignore for now
            }
            SBlockPickRequest::PACKET_ID => {
                self.handle_block_pick_request(player, SBlockPickRequest::read(reader)?)
                    .await;
            }
            SMobEquipment::PACKET_ID => {
                self.handle_mob_equipment(player, SMobEquipment::read(reader)?)
                    .await;
            }
            _ => {
                warn!("Bedrock: Received Unknown Game packet: {}", packet.id);
            }
        }
        Ok(())
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
                        UNIX_EPOCH.elapsed().unwrap().as_millis() as u64,
                    ),
                    RakReliability::Unreliable,
                )
                .await;
            }
            SNewIncomingConnection::PACKET_ID => {
                self.handle_new_incoming_connection(&SNewIncomingConnection::read(reader)?);
            }
            SConnectedPing::PACKET_ID => {
                self.handle_connected_ping(SConnectedPing::read(reader)?)
                    .await;
            }
            SDisconnect::PACKET_ID => {
                self.close().await;
            }
            _ => {
                warn!("Bedrock: Received Unknown RakNet Online packet: {packet_id}");
            }
        }
        Ok(())
    }

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
            SUnconnectedPing::PACKET_ID => {
                Self::handle_unconnected_ping(
                    server,
                    SUnconnectedPing::read(payload)?,
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
                            warn!("Failed to decode packet from client: {err}");
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
        } else if tokio::runtime::Handle::try_current().is_ok() {
            Some(self.tasks.spawn(task))
        } else {
            warn!("No Tokio runtime in current thread; running task on dedicated runtime thread");
            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to build fallback runtime");
                rt.block_on(async move {
                    let _ = task.await;
                });
            });
            None
        }
    }
}
