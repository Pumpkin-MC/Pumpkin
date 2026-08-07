use std::{
    io::{Cursor, Error},
    net::SocketAddr,
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

pub async fn handle_packet(
    server: &Server,
    packet_id: u8,
    payload: &[u8],
    address: SocketAddr,
    socket: &UdpSocket,
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

    let player_count = server
        .get_status()
        .lock()
        .await
        .status_response
        .players
        .as_ref()
        .map_or(0, |players| players.online) as i32;
    let port = server.advanced_config.networking.bedrock.address.port();
    let server_info = ServerInfo {
        edition: "MCPE",
        motd_line_1: server.advanced_config.networking.bedrock.motd.clone(),
        protocol_version: CURRENT_BEDROCK_MC_PROTOCOL,
        version_name: CURRENT_BEDROCK_MC_VERSION,
        player_count,
        max_player_count: server.advanced_config.networking.bedrock.max_players,
        server_unique_id: server.server_guid,
        motd_line_2: server.basic_config.default_level_name.clone(),
        game_mode: server.defaultgamemode.lock().await.gamemode.to_str(),
        game_mode_numeric: 1,
        port_ipv4: port,
        port_ipv6: 19133,
    };
    let pong = CUnconnectedPong::new(time, server.server_guid, server_info.to_string());
    let mut data = vec![CUnconnectedPong::PACKET_ID as u8];
    pong.write_packet(&mut data)?;
    let _ = socket.send_to(&data, address).await;
    Ok(())
}
