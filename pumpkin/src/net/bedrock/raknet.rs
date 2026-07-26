use super::BedrockClient;
use crate::net::DisconnectReason;
use crate::server::Server;
use bytes::Bytes;
use pumpkin_protocol::BClientPacket;
use pumpkin_protocol::PacketDecodeError;
use pumpkin_protocol::RawPacket;
use pumpkin_protocol::bedrock::MTU;
use pumpkin_protocol::bedrock::RAKNET_ACK;
use pumpkin_protocol::bedrock::RAKNET_GAME_PACKET;
use pumpkin_protocol::bedrock::RAKNET_NACK;
use pumpkin_protocol::bedrock::RAKNET_VALID;
use pumpkin_protocol::bedrock::RakReliability;
use pumpkin_protocol::bedrock::SPLIT_FRAME_MAX_CONTENT;
use pumpkin_protocol::bedrock::UDP_HEADER_SIZE;
use pumpkin_protocol::bedrock::ack::Acknowledge;
use pumpkin_protocol::bedrock::client::raknet::connection::CAlreadyConnected;
use pumpkin_protocol::bedrock::client::raknet::connection::CConnectionBanned;
use pumpkin_protocol::bedrock::client::raknet::connection::CConnectionRequestAccepted;
use pumpkin_protocol::bedrock::client::raknet::connection::CNoFreeIncomingConnections;
use pumpkin_protocol::bedrock::frame_set::Frame;
use pumpkin_protocol::bedrock::frame_set::FrameSet;
use pumpkin_protocol::bedrock::server::raknet::connection::SConnectedPing;
use pumpkin_protocol::bedrock::server::raknet::connection::SConnectionLost;
use pumpkin_protocol::bedrock::server::raknet::connection::SConnectionRequest;
use pumpkin_protocol::bedrock::server::raknet::connection::SDisconnect;
use pumpkin_protocol::bedrock::server::raknet::connection::SNewIncomingConnection;
use pumpkin_protocol::bedrock::server::raknet::open_connection::SOpenConnectionRequest1;
use pumpkin_protocol::bedrock::server::raknet::open_connection::SOpenConnectionRequest2;
use pumpkin_protocol::bedrock::server::raknet::unconnected_ping::SUnconnectedPing;
use pumpkin_protocol::bedrock::server::raknet::unconnected_ping::SUnconnectedPingOpenConnections;
use pumpkin_protocol::codec::u24;
use pumpkin_protocol::packet::Packet;
use pumpkin_protocol::serial::PacketRead;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::io::Cursor;
use std::io::Error;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::net::SocketAddrV4;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::UNIX_EPOCH;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tracing::debug;
use tracing::error;
use tracing::warn;

impl BedrockClient {
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
            SDisconnect::PACKET_ID | SConnectionLost::PACKET_ID => {
                self.close().await;
            }
            _ => {
                warn!("Bedrock: Received Unknown RakNet Online packet: {packet_id}");
            }
        }
        Ok(())
    }
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
                let mut banned_ips = server.data.banned_ip_list.write().await;
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
                let status = server.get_status().lock().await;
                status
                    .status_response
                    .players
                    .as_ref()
                    .map_or(0, |p| p.online) as u32
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
}
