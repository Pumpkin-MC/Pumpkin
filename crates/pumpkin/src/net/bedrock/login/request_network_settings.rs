#[allow(clippy::wildcard_imports)]
use super::*;

impl BedrockClient {
    pub async fn handle_request_network_settings(
        &self,
        packet: SRequestNetworkSettings,
        server: &Server,
    ) {
        if packet.client_network_version < CURRENT_BEDROCK_MC_PROTOCOL as i32 {
            self.send_packet(&CPlayStatus::OutdatedClient).await;
            return;
        } else if packet.client_network_version > CURRENT_BEDROCK_MC_PROTOCOL as i32 {
            self.send_packet(&CPlayStatus::OutdatedServer).await;
            return;
        }

        self.version.store(BedrockMinecraftVersion::from_protocol(
            packet.client_network_version as u32,
        ));

        let compression = server
            .advanced_config
            .networking
            .bedrock
            .compression
            .info
            .clone();

        self.send_packet(&CNetworkSettings {
            compression_threshold: compression.threshold as u16,
            compression_algorithm: 0,
            client_throttle_enabled: false,
            client_throttle_threshold: 0,
            client_throttle_scalar: 0.0,
        })
        .await;
        self.set_compression(compression).await;
    }
}
