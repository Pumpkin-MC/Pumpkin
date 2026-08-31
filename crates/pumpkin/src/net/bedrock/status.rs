use std::{
    io::{Cursor, Error},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    sync::Arc,
};

use crate::server::Server;
use bytes::Bytes;
use pumpkin_protocol::{
    BClientPacket,
    bedrock::status::{
        CIncompatibleProtocolVersion, CUnconnectedPong, OFFLINE_MESSAGE_MAGIC, SUnconnectedPing,
        SUnconnectedPingOpenConnections, ServerInfo,
    },
    packet::Packet,
    serial::PacketRead,
};
use pumpkin_world::{CURRENT_BEDROCK_MC_PROTOCOL, CURRENT_BEDROCK_MC_VERSION};
use tokio::{
    net::UdpSocket,
    sync::{Mutex, mpsc},
};
use tracing::trace;

pub struct StatusResponder {
    ipv4: Arc<UdpSocket>,
    ipv6: UdpSocket,
    ipv4_port: u16,
    ipv6_port: u16,
    ice_packets: mpsc::Sender<(Bytes, SocketAddr)>,
}

/// The WebRTC side of the UDP socket shared with Bedrock server-list status.
pub struct IceSocket {
    socket: Arc<UdpSocket>,
    packets: Mutex<mpsc::Receiver<(Bytes, SocketAddr)>>,
}

impl StatusResponder {
    pub async fn bind(address: SocketAddr) -> Result<(Self, IceSocket), Error> {
        let ipv4_ip = match address.ip() {
            IpAddr::V4(ip) => ip,
            IpAddr::V6(_) => Ipv4Addr::UNSPECIFIED,
        };
        let ipv4_port = address.port();
        let ipv6_port = ipv4_port.saturating_add(1);
        let ipv4 = Arc::new(UdpSocket::bind((ipv4_ip, ipv4_port)).await?);
        let (ice_packets, packets) = mpsc::channel(1024);
        Ok((
            Self {
                ipv4: ipv4.clone(),
                ipv6: UdpSocket::bind((Ipv6Addr::UNSPECIFIED, ipv6_port)).await?,
                ipv4_port,
                ipv6_port,
                ice_packets,
            },
            IceSocket {
                socket: ipv4,
                packets: Mutex::new(packets),
            },
        ))
    }

    pub fn local_addrs(&self) -> Result<(SocketAddr, SocketAddr), Error> {
        Ok((self.ipv4.local_addr()?, self.ipv6.local_addr()?))
    }

    pub async fn receive(&self, server: &Server) -> Result<(), Error> {
        let mut ipv4_buffer = [0; 2048];
        let mut ipv6_buffer = [0; 64];
        tokio::select! {
            result = self.ipv4.recv_from(&mut ipv4_buffer) => {
                let (length, client) = result?;
                let packet = &ipv4_buffer[..length];
                if is_status_packet(packet) {
                    trace!(%client, length, "Received Bedrock server-list status ping");
                    self.respond(server, &self.ipv4, packet, client).await
                } else if let Some(client_protocol) = raknet_protocol_version(packet) {
                    self.reject_legacy_raknet(server, &self.ipv4, client, client_protocol).await
                } else {
                    trace!(
                        %client,
                        length,
                        kind = ice_packet_kind(packet),
                        "Received Bedrock ICE datagram"
                    );
                    if self.ice_packets.try_send((Bytes::copy_from_slice(packet), client)).is_err() {
                        trace!(%client, "Dropped Bedrock ICE datagram because its queue is unavailable");
                    }
                    Ok(())
                }
            }
            result = self.ipv6.recv_from(&mut ipv6_buffer) => {
                let (length, client) = result?;
                trace!(%client, length, "Received Bedrock IPv6 server-list status packet");
                let packet = &ipv6_buffer[..length];
                if let Some(client_protocol) = raknet_protocol_version(packet) {
                    self.reject_legacy_raknet(server, &self.ipv6, client, client_protocol).await
                } else {
                    self.respond(server, &self.ipv6, packet, client).await
                }
            }
        }
    }

    async fn reject_legacy_raknet(
        &self,
        server: &Server,
        socket: &UdpSocket,
        client: SocketAddr,
        client_protocol: u8,
    ) -> Result<(), Error> {
        let server_protocol = client_protocol.saturating_add(1);
        let packet = CIncompatibleProtocolVersion::new(server_protocol, server.server_guid);
        let mut response = vec![CIncompatibleProtocolVersion::PACKET_ID as u8];
        packet.write_packet(&mut response)?;
        socket.send_to(&response, client).await?;
        trace!(%client, client_protocol, server_protocol, "Rejected unsupported Bedrock RakNet connection");
        Ok(())
    }

    async fn respond(
        &self,
        server: &Server,
        socket: &UdpSocket,
        packet: &[u8],
        client: SocketAddr,
    ) -> Result<(), Error> {
        let Some((&packet_id, payload)) = packet.split_first() else {
            return Ok(());
        };
        handle_packet(
            server,
            packet_id,
            payload,
            client,
            socket,
            self.ipv4_port,
            self.ipv6_port,
        )
        .await
    }
}

fn is_status_packet(packet: &[u8]) -> bool {
    matches!(packet.first(), Some(&id) if id == SUnconnectedPing::PACKET_ID as u8
        || id == SUnconnectedPingOpenConnections::PACKET_ID as u8)
        && packet.get(9..25) == Some(OFFLINE_MESSAGE_MAGIC.as_slice())
}

fn raknet_protocol_version(packet: &[u8]) -> Option<u8> {
    (packet.first() == Some(&0x05) && packet.get(1..17) == Some(OFFLINE_MESSAGE_MAGIC.as_slice()))
        .then(|| packet.get(17).copied())
        .flatten()
}

fn ice_packet_kind(packet: &[u8]) -> &'static str {
    if packet.len() >= 20 && packet.get(4..8) == Some(&[0x21, 0x12, 0xa4, 0x42]) {
        "STUN"
    } else if matches!(packet.first(), Some(20..=63)) {
        "DTLS"
    } else {
        "unknown"
    }
}

impl IceSocket {
    pub fn local_addr(&self) -> Result<SocketAddr, Error> {
        self.socket.local_addr()
    }

    pub async fn recv_from(&self, buffer: &mut [u8]) -> Result<(usize, SocketAddr), Error> {
        let (packet, address) = self.packets.lock().await.recv().await.ok_or_else(|| {
            Error::new(std::io::ErrorKind::BrokenPipe, "Bedrock UDP socket closed")
        })?;
        let length = buffer.len().min(packet.len());
        buffer[..length].copy_from_slice(&packet[..length]);
        Ok((length, address))
    }

    pub async fn send_to(&self, buffer: &[u8], target: SocketAddr) -> Result<usize, Error> {
        self.socket.send_to(buffer, target).await
    }
}

pub async fn handle_packet(
    server: &Server,
    packet_id: u8,
    payload: &[u8],
    client: SocketAddr,
    socket: &UdpSocket,
    ipv4_port: u16,
    ipv6_port: u16,
) -> Result<(), Error> {
    let (time, magic) = match i32::from(packet_id) {
        SUnconnectedPing::PACKET_ID => {
            let packet = SUnconnectedPing::read(&mut Cursor::new(payload))?;
            (packet.time, packet.magic)
        }
        SUnconnectedPingOpenConnections::PACKET_ID => {
            let packet = SUnconnectedPingOpenConnections::read(&mut Cursor::new(payload))?;
            (packet.time, packet.magic)
        }
        _ => return Ok(()),
    };
    if magic != OFFLINE_MESSAGE_MAGIC {
        return Ok(());
    }

    let players = server
        .get_status()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .status_response
        .players
        .as_ref()
        .map_or(0, |players| players.online) as i32;
    let game_mode = server
        .defaultgamemode
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .gamemode;
    let info = ServerInfo {
        motd: &server.advanced_config.networking.bedrock.motd,
        protocol: CURRENT_BEDROCK_MC_PROTOCOL,
        version: CURRENT_BEDROCK_MC_VERSION,
        players,
        max_players: server.advanced_config.networking.bedrock.max_players,
        server_guid: server.server_guid,
        level_name: &server.basic_config.default_level_name,
        game_mode: game_mode.to_str(),
        game_mode_id: 1,
        ipv4_port,
        ipv6_port,
    };
    let pong = CUnconnectedPong::new(time, server.server_guid, info.to_string());
    let mut response = vec![CUnconnectedPong::PACKET_ID as u8];
    pong.write_packet(&mut response)?;
    socket.send_to(&response, client).await?;
    trace!(
        %client,
        players,
        max_players = server.advanced_config.networking.bedrock.max_players,
        protocol = CURRENT_BEDROCK_MC_PROTOCOL,
        response_length = response.len(),
        "Sent Bedrock server-list status pong"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distinguishes_raknet_status_from_stun() {
        let mut ping = [0; 33];
        ping[0] = SUnconnectedPing::PACKET_ID as u8;
        ping[9..25].copy_from_slice(&OFFLINE_MESSAGE_MAGIC);
        assert!(is_status_packet(&ping));

        let mut stun_success = [0; 32];
        stun_success[..2].copy_from_slice(&[0x01, 0x01]);
        stun_success[4..8].copy_from_slice(&[0x21, 0x12, 0xa4, 0x42]);
        assert!(!is_status_packet(&stun_success));
        assert_eq!(ice_packet_kind(&stun_success), "STUN");

        let mut open_connection = [0; 18];
        open_connection[0] = 0x05;
        open_connection[1..17].copy_from_slice(&OFFLINE_MESSAGE_MAGIC);
        open_connection[17] = 11;
        assert_eq!(raknet_protocol_version(&open_connection), Some(11));
    }

    #[tokio::test]
    async fn ice_socket_uses_the_shared_udp_port() {
        let server = Arc::new(UdpSocket::bind("127.0.0.1:0").await.unwrap());
        let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let (sender, receiver) = mpsc::channel(1);
        let ice = IceSocket {
            socket: server.clone(),
            packets: Mutex::new(receiver),
        };
        sender
            .send((Bytes::from_static(b"request"), client.local_addr().unwrap()))
            .await
            .unwrap();

        let mut request = [0; 16];
        let (length, address) = ice.recv_from(&mut request).await.unwrap();
        assert_eq!(&request[..length], b"request");
        assert_eq!(address, client.local_addr().unwrap());

        ice.send_to(b"response", client.local_addr().unwrap())
            .await
            .unwrap();
        let mut response = [0; 16];
        let (length, address) = client.recv_from(&mut response).await.unwrap();
        assert_eq!(&response[..length], b"response");
        assert_eq!(address, server.local_addr().unwrap());
    }
}
