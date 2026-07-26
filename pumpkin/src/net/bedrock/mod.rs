use crate::entity::player::Player;
use crate::net::DisconnectReason;
use crate::net::GameProfile;
use crate::net::PlayerConfig;
use crate::plugin::api::events::world::chunk_send::ChunkSend;
use crate::server::Server;
use arc_swap::ArcSwap;
use bytes::Bytes;
use crossbeam::atomic::AtomicCell;
use pumpkin_config::networking::compression::CompressionInfo;
use pumpkin_protocol::BClientPacket;
use pumpkin_protocol::RawPacket;
use pumpkin_protocol::bedrock::RAKNET_ACK;
use pumpkin_protocol::bedrock::RakReliability;
use pumpkin_protocol::bedrock::SubClient;
use pumpkin_protocol::bedrock::ack::Acknowledge;
use pumpkin_protocol::bedrock::client::disconnect_player::CDisconnectPlayer;
use pumpkin_protocol::bedrock::client::level_chunk::CLevelChunk;
use pumpkin_protocol::bedrock::client::raknet::connection::CDisconnect;
use pumpkin_protocol::bedrock::frame_set::Frame;
use pumpkin_protocol::bedrock::frame_set::FrameSet;
use pumpkin_protocol::bedrock::packet_decoder::UDPNetworkDecoder;
use pumpkin_protocol::bedrock::packet_encoder::UDPNetworkEncoder;
use pumpkin_protocol::bedrock::server::login::ClientData;
use pumpkin_protocol::packet::Packet;
use pumpkin_util::version::BedrockMinecraftVersion;
use pumpkin_world::level::SyncChunk;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Error;
use std::io::Write;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU16;
use std::sync::atomic::AtomicU32;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::debug;
use tracing::error;
use tracing::warn;

pub mod connection;
mod dispatch;
pub mod login;
pub mod open_connection;
pub mod play;
mod raknet;
pub mod unconnected;

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

    pub const fn with_completion(data: Bytes, completion: oneshot::Sender<()>) -> Self {
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
    rt_handle: tokio::runtime::Handle,
    /// FIFO queue of serialized packets to send to the network.
    outgoing_packet_queue_send: Sender<OutgoingPacket>,
    /// FIFO queue of serialized packets to send to the network.
    outgoing_packet_queue_recv: Mutex<Option<Receiver<OutgoingPacket>>>,

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
    pending_profile: Mutex<Option<(GameProfile, PlayerConfig)>>,
}

impl BedrockClient {
    #[must_use]
    pub fn new(
        socket: Arc<UdpSocket>,
        address: SocketAddr,
        be_clients: Arc<Mutex<HashMap<SocketAddr, Arc<Self>>>>,
    ) -> Self {
        let (send, recv) = tokio::sync::mpsc::channel(4096);
        let (incoming_send, incoming_recv) = tokio::sync::mpsc::channel(4096);
        let rt_handle = tokio::runtime::Handle::current();
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
            rt_handle,
            outgoing_packet_queue_send: send,
            outgoing_packet_queue_recv: Mutex::new(Some(recv)),
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
            pending_profile: Mutex::new(None),
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
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(100));

            while !client.close_token.is_cancelled() {
                let mut packet = tokio::select! {
                    () = client.close_token.cancelled() => break,
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

                // Encrypt the packet payload if encryption is enabled.
                if packet.data.len() > 1 && packet.data[0] == 0xfe {
                    let mut encoder = client.network_writer.write().await;
                    if let Some(encryptor) = encoder.encryptor_mut() {
                        let mut data_to_encrypt = packet.data[1..].to_vec();
                        encryptor.encrypt(&mut data_to_encrypt);
                        let mut encrypted_payload = Vec::with_capacity(1 + data_to_encrypt.len());
                        encrypted_payload.push(0xfe);
                        encrypted_payload.extend_from_slice(&data_to_encrypt);
                        packet.data = encrypted_payload.into();
                    }
                }

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
            let event = ChunkSend::new(player.world(), chunk.clone());
            let event = server.plugin_manager.fire(event).await;
            if !event.cancelled {
                valid_chunks.push(chunk.clone());
            }
        }

        if valid_chunks.is_empty() {
            return;
        }

        let mut serialize_tasks = Vec::with_capacity(valid_chunks.len());
        for chunk in valid_chunks {
            serialize_tasks.push(tokio::task::spawn_blocking(move || {
                let mut packet_payload = Vec::new();
                let packet = CLevelChunk {
                    dimension: 0,
                    cache_enabled: false,
                    chunk: &chunk,
                };
                packet
                    .write_packet(&mut packet_payload)
                    .map(|()| packet_payload)
            }));
        }

        let mut encoded_payloads = Vec::with_capacity(serialize_tasks.len());
        for task in serialize_tasks {
            match task.await {
                Ok(Ok(payload)) => encoded_payloads.push(payload),
                Ok(Err(e)) => error!("Failed to serialize Bedrock chunk: {:?}", e),
                Err(e) => error!("Join error in Bedrock chunk serialization: {:?}", e),
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
                    .outgoing_packet_queue_send
                    .send(OutgoingPacket::with_completion(payload, tx))
                    .await
                {
                    if !self.is_closed() {
                        error!("Failed to add packet to the outgoing packet queue: {err}");
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
    pub async fn close(&self) {
        if self.is_closed() {
            return;
        }
        self.close_token.cancel();

        self.send_framed_packet(&CDisconnect, RakReliability::Unreliable)
            .await;

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
    pub async fn await_close_interrupt(&self) {
        self.close_token.cancelled().await;
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
}
