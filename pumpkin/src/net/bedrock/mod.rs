pub mod play;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    io::{Cursor, Error, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU16, AtomicU32, Ordering},
    },
};

use tracing::{debug, error, warn};

use bytes::Bytes;
use pumpkin_config::networking::compression::CompressionInfo;
use pumpkin_protocol::{
    BClientPacket, PacketDecodeError, RawPacket,
    bedrock::{
        MTU, RAKNET_ACK, RAKNET_GAME_PACKET, RAKNET_NACK, RakReliability, SubClient,
        ack::Acknowledge,
        client::disconnect_player::CDisconnectPlayer,
        frame_set::{Frame, FrameSet},
        packet_decoder::UDPNetworkDecoder,
        packet_encoder::UDPNetworkEncoder,
        server::{
            client_cache_status::SClientCacheStatus,
            command_request::SCommandRequest,
            container_close::SContainerClose,
            interaction::SInteraction,
            login::SLogin,
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
    sync::Mutex,
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};

pub mod connection;
pub mod login;
pub mod open_connection;
pub mod unconnected;
use crate::{entity::player::Player, net::DisconnectReason, server::Server};

pub struct BedrockClient {
    socket: Arc<UdpSocket>,
    /// The client's IP address.
    pub address: SocketAddr,
    pub player: Mutex<Option<Arc<Player>>>,
    /// All Bedrock clients
    /// This list is used to remove the client if the connection gets closed
    pub be_clients: Arc<Mutex<HashMap<SocketAddr, Arc<Self>>>>,

    tasks: TaskTracker,
    outgoing_packet_queue_send: Sender<Bytes>,
    /// A queue of serialized packets to send to the network
    outgoing_packet_queue_recv: Mutex<Option<Receiver<Bytes>>>,

    /// The packet encoder for outgoing packets.
    network_writer: Arc<Mutex<UDPNetworkEncoder>>,
    /// The packet decoder for incoming packets.
    network_reader: Mutex<UDPNetworkDecoder>,

    _use_frame_sets: AtomicBool,
    output_sequence_number: AtomicU32,
    output_reliable_number: AtomicU32,
    output_split_number: AtomicU16,
    output_sequenced_index: AtomicU32,
    output_ordered_index: AtomicU32,
    /// An notifier that is triggered when this client is closed.
    close_token: CancellationToken,
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
}

impl BedrockClient {
    #[must_use]
    pub fn new(
        socket: Arc<UdpSocket>,
        address: SocketAddr,
        be_clients: Arc<Mutex<HashMap<SocketAddr, Arc<Self>>>>,
    ) -> Self {
        let (send, recv) = tokio::sync::mpsc::channel(128);
        Self {
            socket,
            player: Mutex::new(None),
            address,
            be_clients,
            network_writer: Arc::new(Mutex::new(UDPNetworkEncoder::new())),
            network_reader: Mutex::new(UDPNetworkDecoder::new()),
            tasks: TaskTracker::new(),
            outgoing_packet_queue_send: send,
            outgoing_packet_queue_recv: Mutex::new(Some(recv)),
            _use_frame_sets: AtomicBool::new(false),
            output_sequence_number: AtomicU32::new(0),
            output_reliable_number: AtomicU32::new(0),
            output_split_number: AtomicU16::new(0),
            output_sequenced_index: AtomicU32::new(0),
            output_ordered_index: AtomicU32::new(0),
            compounds: Arc::new(Mutex::new(HashMap::new())),
            close_token: CancellationToken::new(),
            received_sequences: Mutex::new(HashSet::new()),
            pending_acks: Mutex::new(Vec::new()),
            unacked_outgoing_frames: Mutex::new(HashMap::new()),
            expected_order_index: Mutex::new(HashMap::new()),
            highest_sequence_index: Mutex::new(HashMap::new()),
            ordered_queues: Mutex::new(HashMap::new()),
            //input_sequence_number: AtomicU32::new(0),
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
                tokio::select! {
                    _ = interval.tick() => {
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
                            let unacked = client.unacked_outgoing_frames.lock().await;
                            for (seq, (id, data, timestamp)) in unacked.iter() {
                                if now.duration_since(*timestamp) > std::time::Duration::from_secs(1) {
                                    resend.push((*seq, *id, data.clone()));
                                }
                            }
                        }

                        for (seq, id, data) in resend {
                             debug!("Resending reliable sequence {} (ID: {})", seq, id);
                             // Update timestamp
                             if let Some(entry) = client.unacked_outgoing_frames.lock().await.get_mut(&seq) {
                                 entry.2 = now;
                             }
                             if let Err(err) = client.network_writer.lock().await.write_packet(&data, client.address, &client.socket).await {
                                 warn!("Failed to resend packet for sequence {}: {}", seq, err);
                             }
                        }
                    }
                    recv_result = packet_receiver.recv() => {
                        let Some(packet_data) = recv_result else {
                            break;
                        };

                        if let Err(err) = client.network_writer
                            .lock()
                            .await
                            .write_packet(&packet_data, client.address, &client.socket)
                            .await
                            && !client.close_token.is_cancelled() {
                                warn!("Failed to send packet to client: {err}",);
                                client.close_token.cancel();
                                break;
                            }
                    }
                }
            }
        });
    }

    pub async fn process_packet(self: &Arc<Self>, server: &Arc<Server>, packet: Cursor<Vec<u8>>) {
        let packet = self.get_packet_payload(packet).await;
        if let Some(packet) = packet
            && let Err(error) = self.handle_packet_payload(server, packet).await
        {
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
            .lock()
            .await
            .set_compression((compression.threshold as usize, compression.level));
    }

    pub async fn kick(&self, reason: DisconnectReason, message: String) {
        self.send_game_packet(&CDisconnectPlayer::new(reason as i32, message))
            .await;
        self.close().await;
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
            if !self.is_closed() {
                error!("Failed to add packet to the outgoing packet queue for client: {err}");
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

        // TODO
        self.network_writer.lock().await.write_game_packet(
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
                self.send_framed_packet_data(packet_buf, RakReliability::ReliableOrdered)
                    .await;
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

        let count = if packet_buf.len() > MTU {
            reliability = RakReliability::ReliableOrdered;
            split_id = self.output_split_number.fetch_add(1, Ordering::Relaxed);
            split_size = packet_buf.len().div_ceil(MTU) as u32;
            split_size as usize
        } else {
            1
        };

        if reliability.is_ordered() {
            order_index = self.output_ordered_index.fetch_add(1, Ordering::Relaxed);
        }

        for i in 0..count {
            let end = if i + 1 == count {
                packet_buf.len() % MTU
            } else {
                MTU
            };
            let chunk = &packet_buf[i * MTU..i * MTU + end];

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
            .lock()
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
        self.close_token.cancel();
        self.tasks.close();
        self.tasks.wait().await;
        self.be_clients.lock().await.remove(&self.address);

        if let Some(player) = self.player.lock().await.as_ref() {
            player.remove().await;
        }
    }

    pub fn is_closed(&self) -> bool {
        self.close_token.is_cancelled()
    }

    pub async fn send_acknowledgement(&self, ack: &Acknowledge, id: u8) -> Result<(), Error> {
        let mut packet_buf = Vec::new();
        ack.write(&mut packet_buf, id)?;

        if let Err(err) = self
            .network_writer
            .lock()
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
            0x80..0x8d => {
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
        let unacked = self.unacked_outgoing_frames.lock().await;
        for seq in &nack.sequences {
            if let Some((_id, data, _timestamp)) = unacked.get(seq)
                && let Err(err) = self
                    .network_writer
                    .lock()
                    .await
                    .write_packet(data, self.address, &self.socket)
                    .await
            {
                warn!("Failed to resend packet for sequence {}: {}", seq, err);
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
        let mut payload = Cursor::new(payload);
        let id = u8::read(&mut payload)?;
        self.handle_raknet_packet(server, i32::from(id), payload)
            .await
    }

    async fn handle_game_packet(
        self: &Arc<Self>,
        server: &Arc<Server>,
        packet: RawPacket,
    ) -> Result<(), Error> {
        let packet_id = packet.id;
        let payload = &mut Cursor::new(&packet.payload);
        let result = match packet.id {
            SRequestNetworkSettings::PACKET_ID => {
                self.handle_request_network_settings(SRequestNetworkSettings::read(payload)?)
                    .await;
                Ok(())
            }
            SLogin::PACKET_ID => {
                self.handle_login(SLogin::read(payload)?, server).await;
                Ok(())
            }
            SClientCacheStatus::PACKET_ID => {
                // TODO
                Ok(())
            }
            SResourcePackResponse::PACKET_ID => {
                self.handle_resource_pack_response(SResourcePackResponse::read(payload)?, server)
                    .await;
                Ok(())
            }
            _ => {
                let player_lock = self.player.lock().await;
                if let Some(player) = player_lock.as_ref() {
                    self.handle_play_packet(player, server, packet).await
                } else {
                    debug!(
                        "Received game packet {} before player was initialized",
                        packet.id
                    );
                    Ok(())
                }
            }
        };
        result.map_err(|e| Error::new(e.kind(), format!("Game packet {packet_id}: {e}")))
    }

    pub async fn handle_play_packet(
        &self,
        player: &Arc<Player>,
        server: &Arc<Server>,
        packet: RawPacket,
    ) -> Result<(), Error> {
        let reader = &mut &packet.payload[..];
        match packet.id {
            SPlayerAuthInput::PACKET_ID => {
                self.player_pos_update(player, SPlayerAuthInput::read(reader)?, server)
                    .await;
            }
            SRequestChunkRadius::PACKET_ID => {
                self.handle_request_chunk_radius(player, SRequestChunkRadius::read(reader)?)
                    .await;
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
            _ => {
                warn!("Bedrock: Received Unknown Game packet: {}", packet.id);
            }
        }
        Ok(())
    }

    async fn handle_raknet_packet(
        self: &Arc<Self>,
        server: &Arc<Server>,
        packet_id: i32,
        mut payload: Cursor<Vec<u8>>,
    ) -> Result<(), Error> {
        let reader = &mut payload;
        match packet_id {
            // The client sends this multiple times and some arrive after we already made the connection
            SConnectionRequest::PACKET_ID => (),
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

            RAKNET_GAME_PACKET => {
                let game_packet = self
                    .network_reader
                    .lock()
                    .await
                    .get_game_packet(payload)
                    .map_err(|e| Error::other(e.to_string()))?;

                self.handle_game_packet(server, game_packet).await?;
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
    ) -> Result<(), Error> {
        match i32::from(packet_id) {
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

    pub async fn get_packet_payload(&self, packet: Cursor<Vec<u8>>) -> Option<Bytes> {
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
        } else {
            Some(self.tasks.spawn(task))
        }
    }
}
