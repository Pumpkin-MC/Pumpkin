use pumpkin_config::BASIC_CONFIG;
use pumpkin_protocol::{
    bedrock::{
        client::unconnected_pong::{CUnconnectedPong, ServerInfo},
        server::unconnected_ping::SUnconnectedPing,
    },
    codec::ascii_string::AsciiString,
};

use crate::{net::Client, server::Server};

impl Client {
    pub async fn handle_unconnected_ping(&self, server: &Server, packet: SUnconnectedPing) {
        let motd_string = ServerInfo {
            edition: "MCPE",
            motd_line_1: &BASIC_CONFIG.motd,
            protocol_version: 527,
            version_name: "1.19.1",
            player_count: 1,
            max_player_count: BASIC_CONFIG.max_players,
            server_unique_id: server.server_guid,
            motd_line_2: &BASIC_CONFIG.motd,
            game_mode: "Survival",
            game_mode_numeric: 1,
            port_ipv4: 19132,
            port_ipv6: 19133,
        };
        self.send_packet_now(&CUnconnectedPong::new(
            packet.time,
            server.server_guid,
            packet.magic,
            AsciiString(format!("{motd_string}")),
        ))
        .await;
    }
}
