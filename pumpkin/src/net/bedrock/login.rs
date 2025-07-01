use pumpkin_protocol::bedrock::{
    RakReliability, client::network_settings::CNetworkSettings,
    server::request_network_settings::SRequestNetworkSettings,
};

use crate::net::{Client, bedrock::BedrockClientPlatform};

impl Client {
    pub async fn handle_request_network_settings(
        &self,
        bedrock: &BedrockClientPlatform,
        packet: SRequestNetworkSettings,
    ) {
        dbg!("requested network settings");
        self.protocol_version.store(
            packet.protocol_version,
            std::sync::atomic::Ordering::Relaxed,
        );
        bedrock
            .send_game_packet(
                self,
                &CNetworkSettings::new(0, 0xFF, false, 0, 0.0),
                RakReliability::Unreliable,
            )
            .await;
    }
}
