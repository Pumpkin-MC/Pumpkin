pub mod play;
use std::{
    collections::HashMap,
    io::{Cursor, Error, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU16, AtomicU32, Ordering},
    },
};

use bytes::Bytes;
use pumpkin_config::networking::compression::CompressionInfo;
use pumpkin_protocol::{
    BClientPacket, PacketDecodeError, RawPacket,
    bedrock::{
        MTU, RAKNET_ACK, RAKNET_GAME_PACKET, RAKNET_NACK, RakReliability, SubClient,
        ack::Ack,
        client::disconnect_player::CDisconnectPlayer,
        frame_set::{Frame, FrameSet},
        packet_decoder::UDPNetworkDecoder,
        packet_encoder::UDPNetworkEncoder,
        server::{
            client_cache_status::SClientCacheStatus,
            command_request::SCommandRequest,
            container_close::SContainerClose,
            interaction::SInteraction,
            loading_screen::SLoadingScreen,
            login::SLogin,
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
    sync::{Mutex, Notify},
    task::JoinHandle,
};
use tokio_util::task::TaskTracker;

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
    outgoing_packet_queue_recv: Option<Receiver<Bytes>>,

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
    close_interrupt: Arc<Notify>,

    /// Indicates if the client connection is closed.
    pub closed: Arc<AtomicBool>,

    /// Store Fragments until the packet is complete
    compounds: Arc<Mutex<HashMap<u16, Vec<Option<Frame>>>>>,
    //input_sequence_number: AtomicU32,
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
            outgoing_packet_queue_recv: Some(recv),
            _use_frame_sets: AtomicBool::new(false),
            output_sequence_number: AtomicU32::new(0),
            output_reliable_number: AtomicU32::new(0),
            output_split_number: AtomicU16::new(0),
            output_sequenced_index: AtomicU32::new(0),
            output_ordered_index: AtomicU32::new(0),
            compounds: Arc::new(Mutex::new(HashMap::new())),
            closed: Arc::new(AtomicBool::new(false)),
            close_interrupt: Arc::new(Notify::new()),
            //input_sequence_number: AtomicU32::new(0),
        }
    }

    pub fn start_outgoing_packet_task(&mut self) {
        let mut packet_receiver = self.outgoing_packet_queue_recv.take().unwrap();
        let close_interrupt = self.close_interrupt.clone();
        let closed = self.closed.clone();
        let writer = self.network_writer.clone();
        let addr = self.address;
        let socket = self.socket.clone();
        self.spawn_task(async move {
            while !closed.load(Ordering::Relaxed) {
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

                if let Err(err) = writer
                    .lock()
                    .await
                    .write_packet(&packet_data, addr, &socket)
                    .await
                {
                    // It is expected that the packet will fail if we are closed
                    if !closed.load(Ordering::Relaxed) {
                        log::warn!("Failed to send packet to client: {err}",);
                        // We now need to close the connection to the client since the stream is in an
                        // unknown state
                        close_interrupt.notify_waiters();
                        closed.store(true, Ordering::Relaxed);
                        break;
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
            let _text = format!("Error while reading incoming packet {error}");
            log::error!("Failed to read incoming packet with : {error}");
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
            if !self.closed.load(Ordering::Relaxed) {
                log::error!("Failed to add packet to the outgoing packet queue for client: {err}");
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
        self.network_writer
            .lock()
            .await
            .write_game_packet(
                P::PACKET_ID as u16,
                SubClient::Main,
                SubClient::Main,
                packet_payload.into(),
                write,
            )
            .await
    }

    pub async fn send_offline_packet<P: BClientPacket>(
        packet: &P,
        addr: SocketAddr,
        socket: &UdpSocket,
    ) {
        let mut data = Vec::new();
        let writer = &mut data;
        Self::write_raw_packet(packet, writer).unwrap();
        // We dont care if it works, if not the client will try again!
        let _ = socket.send_to(&data, addr).await;
    }

    pub async fn send_game_packet<P: BClientPacket>(&self, packet: &P) {
        let mut packet_buf = Vec::new();
        self.write_game_packet(packet, &mut packet_buf)
            .await
            .unwrap();
        self.send_framed_packet_data(packet_buf, RakReliability::Unreliable)
            .await;
    }

    pub async fn write_game_packet_to_set<P: BClientPacket>(
        &self,
        packet: &P,
        frame_set: &mut FrameSet,
    ) {
        let mut payload = Vec::new();
        self.write_game_packet(packet, &mut payload).await.unwrap();

        frame_set.frames.push(Frame::new_unreliable(payload));
    }

    pub async fn send_framed_packet<P: BClientPacket>(
        &self,
        packet: &P,
        reliability: RakReliability,
    ) {
        let mut packet_buf = Vec::new();
        Self::write_raw_packet(packet, &mut packet_buf).unwrap();
        self.send_framed_packet_data(packet_buf, reliability).await;
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
        frame_set.sequence = u24(self.output_sequence_number.fetch_add(1, Ordering::Relaxed));
        let mut frame_set_buf = Vec::new();
        frame_set.write_packet_data(&mut frame_set_buf, id).unwrap();

        // I dont know if thats the right place to make encryption & decoding
        if let Err(err) = self
            .network_writer
            .lock()
            .await
            .write_packet(&frame_set_buf, self.address, &self.socket)
            .await
        {
            // It is expected that the packet will fail if we are closed
            if !self.closed.load(Ordering::Relaxed) {
                log::warn!("Failed to send packet to client: {err}");
                // We now need to close the connection to the client since the stream is in an
                // unknown state
                self.closed.store(true, Ordering::Relaxed);
            }
        }
    }

    pub async fn close(&self) {
        self.close_interrupt.notify_waiters();
        self.closed.store(true, Ordering::Relaxed);
        self.tasks.close();
        self.tasks.wait().await;
        self.be_clients.lock().await.remove(&self.address);

        if let Some(player) = self.player.lock().await.as_ref() {
            player.remove().await;
        }
    }

    pub async fn send_ack(&self, ack: &Ack) {
        let mut packet_buf = Vec::new();
        ack.write(&mut packet_buf).unwrap();

        if let Err(err) = self
            .network_writer
            .lock()
            .await
            .write_packet(&packet_buf, self.address, &self.socket)
            .await
        {
            log::warn!("Failed to send packet to client: {err}");
            self.close().await;
        }
    }

    pub async fn handle_packet_payload(
        self: &Arc<Self>,
        server: &Arc<Server>,
        packet: Bytes,
    ) -> Result<(), Error> {
        let reader = &mut Cursor::new(packet);

        match u8::read(reader)? {
            RAKNET_ACK => {
                Self::handle_ack(&Ack::read(reader)?);
            }
            RAKNET_NACK => {
                dbg!("received nack, client is missing packets");
            }
            0x80..0x8d => {
                self.handle_frame_set(server, FrameSet::read(reader)?).await;
            }
            id => {
                log::warn!("Bedrock: Received unknown packet header {id}");
            }
        }
        Ok(())
    }

    fn handle_ack(_ack: &Ack) {}

    async fn handle_frame_set(self: &Arc<Self>, server: &Arc<Server>, frame_set: FrameSet) {
        // TODO: Send all ACKs in short intervals in batches
        self.send_ack(&Ack::new(vec![frame_set.sequence.0])).await;
        // TODO
        for frame in frame_set.frames {
            self.handle_frame(server, frame).await.unwrap();
        }
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

            entry[fragment_index] = Some(frame);

            // Check if all fragments are received
            if entry.iter().any(Option::is_none) {
                return Ok(());
            }

            let mut frames = compounds.remove(&compound_id).unwrap();

            // Safety: We already checked that all frames are Some at this point
            let len = frames
                .iter()
                .map(|frame| unsafe { frame.as_ref().unwrap_unchecked().payload.len() })
                .sum();

            let mut merged = Vec::with_capacity(len);

            for frame in &frames {
                merged.extend_from_slice(unsafe { &frame.as_ref().unwrap_unchecked().payload });
            }

            frame = unsafe { frames[0].take().unwrap_unchecked() };

            frame.payload = merged;
            frame.split_size = 0;
        }

        let mut payload = Cursor::new(frame.payload);
        let id = u8::read(&mut payload)?;
        self.handle_raknet_packet(server, i32::from(id), payload)
            .await
    }

    async fn handle_game_packet(
        self: &Arc<Self>,
        server: &Arc<Server>,
        packet: RawPacket,
    ) -> Result<(), Error> {
        let payload = &mut Cursor::new(&packet.payload);
        match packet.id {
            SRequestNetworkSettings::PACKET_ID => {
                self.handle_request_network_settings(SRequestNetworkSettings::read(payload)?)
                    .await;
            }
            SLogin::PACKET_ID => {
                self.handle_login(SLogin::read(payload)?, server).await;
            }
            SClientCacheStatus::PACKET_ID | SResourcePackResponse::PACKET_ID => {
                // TODO
            }
            _ => {
                self.handle_play_packet(self.player.lock().await.as_ref().unwrap(), server, packet)
                    .await;
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    pub async fn handle_play_packet(
        &self,
        player: &Arc<Player>,
        server: &Arc<Server>,
        packet: RawPacket,
    ) {
        let reader = &mut &packet.payload[..];
        match packet.id {
            SPlayerAuthInput::PACKET_ID => {
                if let Ok(input_packet) = SPlayerAuthInput::read(reader) {
                    self.player_pos_update(player, input_packet).await;
                }
            }
            SLoadingScreen::PACKET_ID => {
                if SLoadingScreen::read(reader).unwrap().is_loading_done() {
                    player.set_client_loaded(true);
                }
            }
            SRequestChunkRadius::PACKET_ID => {
                self.handle_request_chunk_radius(
                    player,
                    SRequestChunkRadius::read(reader).unwrap(),
                )
                .await;
            }
            SInteraction::PACKET_ID => {
                self.handle_interaction(player, SInteraction::read(reader).unwrap())
                    .await;
            }
            SContainerClose::PACKET_ID => {
                self.handle_container_close(player, SContainerClose::read(reader).unwrap())
                    .await;
            }
            SText::PACKET_ID => {
                self.handle_chat_message(server, player, SText::read(reader).unwrap())
                    .await;
            }
            SCommandRequest::PACKET_ID => {
                self.handle_chat_command(player, server, SCommandRequest::read(reader).unwrap())
                    .await;
            }
            _ => {
                log::warn!("Bedrock: Received Unknown Game packet: {}", packet.id);
            }
        }
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
                    .await
                    .map_err(|e| Error::other(e.to_string()))?;

                self.handle_game_packet(server, game_packet).await?;
            }
            _ => {
                log::warn!("Bedrock: Received Unknown RakNet Online packet: {packet_id}");
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
            _ => log::error!("Bedrock: Received Unknown RakNet Offline packet: {packet_id}"),
        }
        Ok(())
    }

    pub async fn await_close_interrupt(&self) {
        self.close_interrupt.notified().await;
    }

    pub async fn get_packet_payload(&self, packet: Cursor<Vec<u8>>) -> Option<Bytes> {
        let mut network_reader = self.network_reader.lock().await;
        tokio::select! {
            () = self.await_close_interrupt() => {
                log::debug!("Canceling player packet processing");
                None
            },
            packet_result = network_reader.get_packet_payload(packet) => {
                match packet_result {
                    Ok(packet) => Some(packet),
                    Err(err) => {
                        if !matches!(err, PacketDecodeError::ConnectionClosed) {
                            log::warn!("Failed to decode packet from client: {err}");
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
        if self.closed.load(Ordering::Relaxed) {
            None
        } else {
            Some(self.tasks.spawn(task))
        }
    }
}
