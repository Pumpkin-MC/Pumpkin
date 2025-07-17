use pumpkin_config::BASIC_CONFIG;
use pumpkin_protocol::{
    bedrock::{
        client::raknet::unconnected_pong::{CUnconnectedPong, ServerInfo},
        server::raknet::unconnected_ping::SUnconnectedPing,
    },
    codec::ascii_string::AsciiString,
};

use crate::{
    net::bedrock::BedrockClient,
    server::{CURRENT_BEDROCK_MC_VERSION, Server},
};

impl BedrockClient {
    pub async fn handle_unconnected_ping(&self, server: &Server, packet: SUnconnectedPing) {
        // TODO
        let player_count = server
            .get_status()
            .lock()
            .await
            .status_response
            .players
            .as_ref()
            .unwrap()
            .online as _;

        let motd_string = ServerInfo {
            edition: "MCPE",
            // TODO The default motd is to long to be displayed completely
            motd_line_1: "Pumpkin Server",
            protocol_version: 819,
            version_name: CURRENT_BEDROCK_MC_VERSION,
            player_count,
            max_player_count: BASIC_CONFIG.max_players,
            server_unique_id: server.server_guid,
            motd_line_2: &BASIC_CONFIG.default_level_name,
            game_mode: server.defaultgamemode.lock().await.gamemode.to_str(),
            game_mode_numeric: 1,
            port_ipv4: 19132,
            port_ipv6: 19133,
        };
        self.send_raknet_packet_now(&CUnconnectedPong::new(
            packet.time,
            server.server_guid,
            packet.magic,
            AsciiString(format!("{motd_string}")),
        ))
        .await;
    }
}
