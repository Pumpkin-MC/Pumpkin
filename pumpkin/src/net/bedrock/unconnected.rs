use pumpkin_config::BASIC_CONFIG;
use pumpkin_protocol::{
    bedrock::{
        client::raknet::unconnected_pong::{CUnconnectedPong, ServerInfo},
        server::raknet::unconnected_ping::SUnconnectedPing,
    },
    codec::ascii_string::AsciiString,
};

use crate::{
    net::{Client, bedrock::BedrockClientPlatform},
    server::Server,
};

impl Client {
    pub async fn handle_unconnected_ping(
        &self,
        bedrock: &BedrockClientPlatform,
        server: &Server,
        packet: SUnconnectedPing,
    ) {
        let motd_string = ServerInfo {
            edition: "MCPE",
            motd_line_1: &BASIC_CONFIG.motd,
            protocol_version: 818,
            version_name: "1.21.90",
            player_count: 1,
            max_player_count: BASIC_CONFIG.max_players,
            server_unique_id: server.server_guid,
            motd_line_2: &BASIC_CONFIG.motd,
            game_mode: "Survival",
            game_mode_numeric: 1,
            port_ipv4: 19132,
            port_ipv6: 19133,
        };
        bedrock
            .send_raknet_packet_now(
                self,
                &CUnconnectedPong::new(
                    packet.time,
                    server.server_guid,
                    packet.magic,
                    AsciiString(format!("{motd_string}")),
                ),
            )
            .await;
    }
}
