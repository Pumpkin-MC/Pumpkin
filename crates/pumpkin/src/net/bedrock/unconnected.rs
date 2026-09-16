use std::net::SocketAddr;

use pumpkin_protocol::bedrock::{
    client::raknet::unconnected_pong::{CUnconnectedPong, ServerInfo},
    server::raknet::unconnected_ping::SUnconnectedPing,
};
use tokio::net::UdpSocket;

use crate::{net::bedrock::BedrockClient, server::Server};
use pumpkin_world::{CURRENT_BEDROCK_MC_PROTOCOL, CURRENT_BEDROCK_MC_VERSION};

impl BedrockClient {
    pub async fn handle_unconnected_ping(
        server: &Server,
        packet: SUnconnectedPing,
        addr: SocketAddr,
        socket: &UdpSocket,
    ) {
        let player_count = server
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
        let ipv4_port = socket.local_addr().map_or(19132, |address| address.port());

        let motd_string = ServerInfo {
            edition: "MCPE",
            motd_line_1: server.advanced_config.networking.bedrock.motd.clone(),
            protocol_version: CURRENT_BEDROCK_MC_PROTOCOL,
            version_name: CURRENT_BEDROCK_MC_VERSION,
            player_count,
            // A large number looks weird on the client worlds window
            max_player_count: server.advanced_config.networking.bedrock.max_players,
            server_unique_id: server.server_guid,
            motd_line_2: server.basic_config.default_level_name.clone(),
            game_mode: game_mode.to_str(),
            game_mode_numeric: 1,
            port_ipv4: ipv4_port,
            port_ipv6: ipv4_port.saturating_add(1),
        };
        Self::send_offline_packet(
            &CUnconnectedPong::new(
                packet.time,
                server.server_guid,
                packet.magic,
                format!("{motd_string}"),
            ),
            addr,
            socket,
        )
        .await;
    }
}
