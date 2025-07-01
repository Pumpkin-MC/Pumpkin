use std::{
    io::{Cursor, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
};

use bytes::Bytes;
use pumpkin_protocol::{
    ClientPacket, PacketDecodeError, PacketEncodeError, RawPacket, ServerPacket,
    bedrock::{
        RAKNET_ACK, RAKNET_GAME_PACKET, RAKNET_NACK, RAKNET_VALID, RakReliability,
        ack::Ack,
        frame_set::{Frame, FrameSet},
        packet_decoder::UDPNetworkDecoder,
        packet_encoder::UDPNetworkEncoder,
        server::{
            raknet::{
                connection::{SConnectionRequest, SDisconnect, SNewIncomingConnection},
                open_connection::{SOpenConnectionRequest1, SOpenConnectionRequest2},
                unconnected_ping::SUnconnectedPing,
            },
            request_network_settings::SRequestNetworkSettings,
        },
    },
    codec::u24::U24,
    packet::Packet,
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError},
};
use std::net::SocketAddr;
use tokio::{net::UdpSocket, sync::Mutex};

pub mod connection;
pub mod login;
pub mod open_connection;
pub mod unconnected;

use crate::{net::Client, server::Server};

pub struct BedrockClientPlatform {
    socket: Arc<UdpSocket>,
    addr: SocketAddr,

    /// The packet encoder for outgoing packets.
    network_writer: Arc<Mutex<UDPNetworkEncoder>>,
    /// The packet decoder for incoming packets.
    network_reader: Mutex<UDPNetworkDecoder>,

    use_frame_sets: AtomicBool,
    output_sequence_number: AtomicU32,
    output_reliable_number: AtomicU32,
    output_sequenced_index: AtomicU32,
    output_ordered_index: AtomicU32,
}

impl BedrockClientPlatform {
    #[must_use]
    pub fn new(socket: Arc<UdpSocket>, addr: SocketAddr) -> Self {
        Self {
            socket,
            addr,
            network_writer: Arc::new(Mutex::new(UDPNetworkEncoder::new())),
            network_reader: Mutex::new(UDPNetworkDecoder::new()),
            use_frame_sets: AtomicBool::new(false),
            output_sequence_number: AtomicU32::new(0),
            output_reliable_number: AtomicU32::new(0),
            output_sequenced_index: AtomicU32::new(0),
            output_ordered_index: AtomicU32::new(0),
        }
    }

    pub async fn process_packet(&self, client: &Client, server: &Server, packet: Cursor<Vec<u8>>) {
        let packet = self.get_packet_payload(client, packet).await;
        if let Some(packet) = packet {
            if let Err(error) = self.handle_packet_payload(client, server, packet).await {
                let _text = format!("Error while reading incoming packet {error}");
                log::error!("Failed to read incoming packet with : {error}");
                //self.kick(TextComponent::text(text)).await;
            }
        }
    }

    pub fn write_raw_packet<P: ClientPacket>(
        packet: &P,
        mut write: impl Write,
    ) -> Result<(), WritingError> {
        write.write_u8(P::PACKET_ID as u8)?;
        packet.write_packet_data(write)
    }

    pub async fn write_game_packet<P: ClientPacket>(
        &self,
        packet: &P,
        write: impl Write,
    ) -> Result<(), WritingError> {
        let mut packet_payload = Vec::new();
        packet.write_packet_data(&mut packet_payload)?;

        // TODO
        self.network_writer
            .lock()
            .await
            .write_game_packet(P::PACKET_ID, 0, 0, packet_payload.into(), write)
            .await
            .unwrap();
        Ok(())
    }

    pub async fn write_packet_data(&self, packet_data: Bytes) -> Result<(), PacketEncodeError> {
        self.network_writer
            .lock()
            .await
            .write_packet(packet_data, self.addr, &self.socket)
            .await
    }

    pub async fn send_raknet_packet_now<P: ClientPacket>(&self, client: &Client, packet: &P) {
        let mut packet_buf = Vec::new();
        let writer = &mut packet_buf;
        Self::write_raw_packet(packet, writer).unwrap();
        self.send_packet_now(client, packet_buf).await;
    }

    pub async fn send_game_packet<P: ClientPacket>(
        &self,
        client: &Client,
        packet: &P,
        reliability: RakReliability,
    ) {
        let mut packet_buf = Vec::new();
        self.write_game_packet(packet, &mut packet_buf)
            .await
            .unwrap();
        self.send_framed_packet_data(client, packet_buf, reliability)
            .await;
    }

    pub async fn send_framed_packet<P: ClientPacket>(
        &self,
        client: &Client,
        packet: &P,
        reliability: RakReliability,
    ) {
        let mut packet_buf = Vec::new();
        Self::write_raw_packet(packet, &mut packet_buf).unwrap();
        self.send_framed_packet_data(client, packet_buf, reliability)
            .await;
    }

    pub async fn send_framed_packet_data(
        &self,
        client: &Client,
        packet_buf: Vec<u8>,
        reliability: RakReliability,
    ) {
        let mut frame_set = FrameSet {
            sequence: U24(self.output_sequence_number.fetch_add(1, Ordering::Relaxed)),
            frames: Vec::with_capacity(1),
        };
        let mut frame = Frame {
            payload: packet_buf.into(),
            reliability,
            ..Default::default()
        };

        if reliability.is_reliable() {
            frame.reliable_number = self.output_reliable_number.fetch_add(1, Ordering::Relaxed);
            if matches!(reliability, RakReliability::ReliableOrdered) {
                //Todo! Check if Fragmenting is needed
            }
        }

        if reliability.is_ordered() {
            frame.order_index = self.output_ordered_index.fetch_add(1, Ordering::Relaxed);
        }

        if reliability.is_sequenced() {
            frame.sequence_index = self.output_sequenced_index.fetch_add(1, Ordering::Relaxed);
        }

        frame_set.frames.push(frame);

        let mut packet_buf = Vec::new();
        frame_set.write_packet_data(&mut packet_buf).unwrap();

        if let Err(err) = self
            .network_writer
            .lock()
            .await
            .write_packet(packet_buf.into(), self.addr, &self.socket)
            .await
        {
            // It is expected that the packet will fail if we are closed
            if !client.closed.load(Ordering::Relaxed) {
                log::warn!("Failed to send packet to client {}: {}", client.id, err);
                // We now need to close the connection to the client since the stream is in an
                // unknown state
                client.close();
            }
        }
    }

    pub async fn send_packet_now(&self, client: &Client, packet: Vec<u8>) {
        if !self.use_frame_sets.load(Ordering::Relaxed) {
            // Sent the packet directly
            if let Err(err) = self
                .network_writer
                .lock()
                .await
                .write_packet(packet.into(), self.addr, &self.socket)
                .await
            {
                // It is expected that the packet will fail if we are closed
                if !client.closed.load(Ordering::Relaxed) {
                    log::warn!("Failed to send packet to client {}: {}", client.id, err);
                    // We now need to close the connection to the client since the stream is in an
                    // unknown state
                    client.close();
                }
            }
        }
    }

    pub async fn handle_packet_payload(
        &self,
        client: &Client,
        server: &Server,
        packet: Bytes,
    ) -> Result<(), ReadingError> {
        let mut payload = &packet[..];

        let Ok(id) = payload.get_u8_be() else {
            return Err(ReadingError::CleanEOF(String::new()));
        };

        let is_valid = id & RAKNET_VALID == RAKNET_VALID;
        if !is_valid {
            // Offline packets just have Packet ID + Payload
            return self
                .handle_offline_packet(client, server, i32::from(id), payload)
                .await;
        }
        self.use_frame_sets.store(true, Ordering::Relaxed);
        let header = id;

        match header {
            RAKNET_ACK => {
                Self::handle_ack(&Ack::read(payload)?);
            }
            RAKNET_NACK => {
                dbg!("received non ack");
            }
            0x80..0x8d => {
                self.handle_frame_set(client, server, FrameSet::read(payload)?)
                    .await;
            }
            _ => {
                log::warn!("Bedrock: Received unknown packet header {header}");
            }
        }
        Ok(())
    }

    fn handle_ack(_ack: &Ack) {
        dbg!("received ack");
    }

    async fn handle_frame_set(&self, client: &Client, server: &Server, frame_set: FrameSet) {
        // TODO: this is bad
        client
            .send_packet_now(&Ack::new(vec![frame_set.sequence.0]))
            .await;
        // TODO
        for frame in frame_set.frames {
            self.handle_frame(client, server, &frame).await.unwrap();
        }
    }

    async fn handle_frame(
        &self,
        client: &Client,
        server: &Server,
        frame: &Frame,
    ) -> Result<(), ReadingError> {
        if frame.split_size > 0 {
            dbg!("oh no, frame is split, TODO");
        }

        dbg!(frame.reliability);

        let mut payload = &frame.payload[..];
        let id = payload.get_u8_be()?;
        self.handle_raknet_packet(client, server, i32::from(id), payload)
            .await
    }

    async fn handle_game_packet(
        &self,
        client: &Client,
        _server: &Server,
        packet: RawPacket,
    ) -> Result<(), ReadingError> {
        let payload = &packet.payload[..];
        match packet.id {
            SRequestNetworkSettings::PACKET_ID => {
                client
                    .handle_request_network_settings(self, SRequestNetworkSettings::read(payload)?)
                    .await;
            }
            _ => {
                log::warn!("Bedrock: Received Unknown Game packet: {}", packet.id);
            }
        }
        Ok(())
    }

    async fn handle_raknet_packet(
        &self,
        client: &Client,
        server: &Server,
        packet_id: i32,
        payload: &[u8],
    ) -> Result<(), ReadingError> {
        match packet_id {
            SConnectionRequest::PACKET_ID => {
                client
                    .handle_connection_request(self, SConnectionRequest::read(payload)?)
                    .await;
            }
            SNewIncomingConnection::PACKET_ID => {
                client.handle_new_incoming_connection(&SNewIncomingConnection::read(payload)?);
            }
            SDisconnect::PACKET_ID => {
                dbg!("Bedrock client disconnected");
                client.close();
            }

            RAKNET_GAME_PACKET => {
                dbg!("game packet");
                dbg!(payload.len());
                let game_packet = self
                    .network_reader
                    .lock()
                    .await
                    .get_game_packet(Cursor::new(payload.to_vec()))
                    .await
                    .unwrap();

                self.handle_game_packet(client, server, game_packet).await?;
            }
            _ => {
                log::warn!("Bedrock: Received Unknown RakNet Online packet: {packet_id}");
            }
        }
        Ok(())
    }

    async fn handle_offline_packet(
        &self,
        client: &Client,
        server: &Server,
        packet_id: i32,
        payload: &[u8],
    ) -> Result<(), ReadingError> {
        match packet_id {
            SUnconnectedPing::PACKET_ID => {
                client
                    .handle_unconnected_ping(self, server, SUnconnectedPing::read(payload)?)
                    .await;
            }
            SOpenConnectionRequest1::PACKET_ID => {
                client
                    .handle_open_connection_1(self, server, SOpenConnectionRequest1::read(payload)?)
                    .await;
            }
            SOpenConnectionRequest2::PACKET_ID => {
                client
                    .handle_open_connection_2(self, server, SOpenConnectionRequest2::read(payload)?)
                    .await;
            }
            _ => {
                log::error!("Bedrock: Received Unknown RakNet Offline packet: {packet_id}");
            }
        }
        Ok(())
    }

    pub async fn get_packet_payload(
        &self,
        client: &Client,
        packet: Cursor<Vec<u8>>,
    ) -> Option<Bytes> {
        let mut network_reader = self.network_reader.lock().await;
        tokio::select! {
            () = client.await_close_interrupt() => {
                log::debug!("Canceling player packet processing");
                None
            },
            packet_result = network_reader.get_packet_payload(packet) => {
                match packet_result {
                    Ok(packet) => Some(packet),
                    Err(err) => {
                        if !matches!(err, PacketDecodeError::ConnectionClosed) {
                            log::warn!("Failed to decode packet from client {}: {}", client.id, err);
                            let _text = format!("Error while reading incoming packet {err}");
                            client.close();
                            //self.kick(client, TextComponent::text(text)).await;
                        }
                        None
                    }
                }
            }
        }
    }
}
