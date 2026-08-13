#[allow(clippy::wildcard_imports)]
use super::*;

impl BedrockClient {
    pub async fn handle_request_network_settings(
        &self,
        packet: SRequestNetworkSettings,
        server: &Server,
    ) {
        if packet.protocol_version < CURRENT_BEDROCK_MC_PROTOCOL as i32 {
            self.send_game_packet(&CPlayStatus::OutdatedClient).await;
            return;
        } else if packet.protocol_version > CURRENT_BEDROCK_MC_PROTOCOL as i32 {
            self.send_game_packet(&CPlayStatus::OutdatedServer).await;
            return;
        }

        self.version.store(BedrockMinecraftVersion::from_protocol(
            packet.protocol_version as u32,
        ));

        let compression = server
            .advanced_config
            .networking
            .bedrock
            .compression
            .info
            .clone();
        self.send_game_packet(&CNetworkSettings::new(
            compression.threshold as u16,
            0,
            false,
            0,
            0.0,
        ))
        .await;
        self.set_compression(compression).await;
    }
}
