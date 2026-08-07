use std::{
    io::{Cursor, Error},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
};

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
use tokio::net::UdpSocket;

use crate::server::Server;

pub struct StatusResponder {
    ipv4: UdpSocket,
    ipv6: UdpSocket,
    ipv4_port: u16,
    ipv6_port: u16,
}

impl StatusResponder {
    pub async fn bind(address: SocketAddr) -> Result<Self, Error> {
        let ipv4_ip = match address.ip() {
            IpAddr::V4(ip) => ip,
            IpAddr::V6(_) => Ipv4Addr::UNSPECIFIED,
        };
        let ipv4_port = address.port();
        let ipv6_port = ipv4_port.saturating_add(1);
        Ok(Self {
            ipv4: UdpSocket::bind((ipv4_ip, ipv4_port)).await?,
            ipv6: UdpSocket::bind((Ipv6Addr::UNSPECIFIED, ipv6_port)).await?,
            ipv4_port,
            ipv6_port,
        })
    }

    pub fn local_addrs(&self) -> Result<(SocketAddr, SocketAddr), Error> {
        Ok((self.ipv4.local_addr()?, self.ipv6.local_addr()?))
    }

    pub async fn receive(&self, server: &Server) -> Result<(), Error> {
        let mut ipv4_buffer = [0; 64];
        let mut ipv6_buffer = [0; 64];
        tokio::select! {
            result = self.ipv4.recv_from(&mut ipv4_buffer) => {
                let (length, client) = result?;
                self.respond(server, &self.ipv4, &ipv4_buffer[..length], client).await
            }
            result = self.ipv6.recv_from(&mut ipv6_buffer) => {
                let (length, client) = result?;
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
    Ok(())
}
