use std::{
    future::Future,
    io::{Cursor, Error, ErrorKind},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    pin::Pin,
    sync::Arc,
};

use bytes::Bytes;
use pumpkin_protocol::{
    BClientPacket,
    bedrock::status::{
        CUnconnectedPong, OFFLINE_MESSAGE_MAGIC, SUnconnectedPing, SUnconnectedPingOpenConnections,
        ServerInfo,
    },
    packet::Packet,
    serial::PacketRead,
};
use pumpkin_world::{CURRENT_BEDROCK_MC_PROTOCOL, CURRENT_BEDROCK_MC_VERSION};
use tokio::{
    net::UdpSocket,
    sync::{Mutex, mpsc},
};
use tracing::{trace, warn};
use webrtc::util::{Conn, Error as WebRtcError};

use crate::server::Server;

// `webrtc::util::Conn` uses `async-trait`. Spell out its object-safe future ABI so
// Pumpkin does not need a direct dependency on the proc macro.
type ConnFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, WebRtcError>> + Send + 'a>>;

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
                self.respond(server, &self.ipv6, &ipv6_buffer[..length], client).await
            }
        }
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
}

impl Conn for IceSocket {
    fn connect<'a, 'async_trait>(&'a self, _address: SocketAddr) -> ConnFuture<'async_trait, ()>
    where
        'a: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async {
            Err(Error::new(
                ErrorKind::Unsupported,
                "the shared Bedrock UDP socket cannot be connected",
            )
            .into())
        })
    }

    fn recv<'a, 'b, 'async_trait>(&'a self, buffer: &'b mut [u8]) -> ConnFuture<'async_trait, usize>
    where
        'a: 'async_trait,
        'b: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move { self.recv_from(buffer).await.map(|(length, _)| length) })
    }

    fn recv_from<'a, 'b, 'async_trait>(
        &'a self,
        buffer: &'b mut [u8],
    ) -> ConnFuture<'async_trait, (usize, SocketAddr)>
    where
        'a: 'async_trait,
        'b: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let (packet, address) =
                self.packets.lock().await.recv().await.ok_or_else(|| {
                    Error::new(ErrorKind::BrokenPipe, "Bedrock UDP socket closed")
                })?;
            let length = buffer.len().min(packet.len());
            buffer[..length].copy_from_slice(&packet[..length]);
            Ok((length, address))
        })
    }

    fn send<'a, 'b, 'async_trait>(&'a self, _buffer: &'b [u8]) -> ConnFuture<'async_trait, usize>
    where
        'a: 'async_trait,
        'b: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async {
            Err(Error::new(
                ErrorKind::NotConnected,
                "the shared Bedrock UDP socket has no default peer",
            )
            .into())
        })
    }

    fn send_to<'a, 'b, 'async_trait>(
        &'a self,
        buffer: &'b [u8],
        target: SocketAddr,
    ) -> ConnFuture<'async_trait, usize>
    where
        'a: 'async_trait,
        'b: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            match self.socket.send_to(buffer, target).await {
                Ok(length) => {
                    trace!(
                        %target,
                        length,
                        kind = ice_packet_kind(buffer),
                        "Sent Bedrock ICE datagram"
                    );
                    Ok(length)
                }
                Err(error) => {
                    warn!(
                        %target,
                        kind = ice_packet_kind(buffer),
                        %error,
                        "Failed to send Bedrock ICE datagram"
                    );
                    Err(error.into())
                }
            }
        })
    }

    fn local_addr(&self) -> Result<SocketAddr, WebRtcError> {
        Ok(self.socket.local_addr()?)
    }

    fn remote_addr(&self) -> Option<SocketAddr> {
        None
    }

    fn close<'a, 'async_trait>(&'a self) -> ConnFuture<'async_trait, ()>
    where
        'a: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async { Ok(()) })
    }

    fn as_any(&self) -> &(dyn std::any::Any + Send + Sync) {
        self
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
        .await
        .status_response
        .players
        .as_ref()
        .map_or(0, |players| players.online) as i32;
    let game_mode = server.defaultgamemode.lock().await.gamemode;
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
        let (length, address) = Conn::recv_from(&ice, &mut request).await.unwrap();
        assert_eq!(&request[..length], b"request");
        assert_eq!(address, client.local_addr().unwrap());

        Conn::send_to(&ice, b"response", client.local_addr().unwrap())
            .await
            .unwrap();
        let mut response = [0; 16];
        let (length, address) = client.recv_from(&mut response).await.unwrap();
        assert_eq!(&response[..length], b"response");
        assert_eq!(address, server.local_addr().unwrap());
    }
}
