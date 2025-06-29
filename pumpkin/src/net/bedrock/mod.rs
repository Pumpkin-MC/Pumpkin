use std::{
    io::{Cursor, Write},
    sync::Arc,
};

use bytes::Bytes;
use pumpkin_protocol::{
    ClientPacket, PacketDecodeError, PacketEncodeError, RawPacket, ServerPacket,
    bedrock::{
        packet_decoder::UDPNetworkDecoder,
        packet_encoder::UDPNetworkEncoder,
        server::{
            open_connection::{SOpenConnectionRequest1, SOpenConnectionRequest2},
            unconnected_ping::SUnconnectedPing,
        },
    },
    packet::Packet,
    ser::{NetworkWriteExt, ReadingError, WritingError},
};
use std::net::SocketAddr;
use tokio::{net::UdpSocket, sync::Mutex};

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
}

impl BedrockClientPlatform {
    #[must_use]
    pub fn new(socket: Arc<UdpSocket>, addr: SocketAddr) -> Self {
        Self {
            socket,
            addr,
            network_writer: Arc::new(Mutex::new(UDPNetworkEncoder::new())),
            network_reader: Mutex::new(UDPNetworkDecoder::new()),
        }
    }

    pub async fn process_packet(&self, client: &Client, server: &Server, packet: Cursor<Vec<u8>>) {
        let packet = self.get_packet(client, packet).await;
        if let Some(packet) = packet {
            if let Err(error) = Self::handle_packet(client, server, &packet).await {
                let _text = format!("Error while reading incoming packet {error}");
                log::error!(
                    "Failed to read incoming packet with id {}: {}",
                    packet.id,
                    error
                );
                //self.kick(TextComponent::text(text)).await;
            }
        }
    }

    pub fn write_packet<P: ClientPacket>(
        packet: &P,
        write: impl Write,
    ) -> Result<(), WritingError> {
        let mut write = write;
        write.write_u8_be(P::PACKET_ID as u8)?;
        packet.write_packet_data(write)
    }

    pub async fn write_packet_data(&self, packet_data: Bytes) -> Result<(), PacketEncodeError> {
        self.network_writer
            .lock()
            .await
            .write_packet(packet_data, self.addr, &self.socket)
            .await
    }

    pub async fn send_packet_now(&self, client: &Client, packet: Vec<u8>) {
        if let Err(err) = self
            .network_writer
            .lock()
            .await
            .write_packet(packet.into(), self.addr, &self.socket)
            .await
        {
            // It is expected that the packet will fail if we are closed
            if !client.closed.load(std::sync::atomic::Ordering::Relaxed) {
                log::warn!("Failed to send packet to client {}: {}", client.id, err);
                // We now need to close the connection to the client since the stream is in an
                // unknown state
                client.close();
            }
        }
    }

    pub async fn handle_packet(
        client: &Client,
        server: &Server,
        packet: &RawPacket,
    ) -> Result<(), ReadingError> {
        let payload = &packet.payload[..];
        match packet.id {
            SUnconnectedPing::PACKET_ID => {
                client
                    .handle_unconnected_ping(server, SUnconnectedPing::read(payload)?)
                    .await;
            }
            SOpenConnectionRequest1::PACKET_ID => {
                client
                    .handle_open_connection_1(server, SOpenConnectionRequest1::read(payload)?)
                    .await;
            }
            SOpenConnectionRequest2::PACKET_ID => {
                client
                    .handle_open_connection_2(server, SOpenConnectionRequest2::read(payload)?)
                    .await;
            }
            _ => {
                log::error!("Failed to handle bedrock client packet id {}", packet.id);
            }
        }
        Ok(())
    }

    pub async fn get_packet(&self, client: &Client, packet: Cursor<Vec<u8>>) -> Option<RawPacket> {
        let mut network_reader = self.network_reader.lock().await;
        tokio::select! {
            () = client.await_close_interrupt() => {
                log::debug!("Canceling player packet processing");
                None
            },
            packet_result = network_reader.get_raw_packet(packet) => {
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
