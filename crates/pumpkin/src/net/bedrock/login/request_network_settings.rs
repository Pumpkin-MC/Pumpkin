#[allow(clippy::wildcard_imports)]
use super::*;

impl BedrockClient {
    pub async fn handle_request_network_settings(
        &self,
        packet: SRequestNetworkSettings,
        server: &Server,
    ) -> bool {
        let status = match packet
            .protocol_version
            .cmp(&(CURRENT_BEDROCK_MC_PROTOCOL as i32))
        {
            std::cmp::Ordering::Less => Some(CPlayStatus::OutdatedClient),
            std::cmp::Ordering::Greater => Some(CPlayStatus::OutdatedServer),
            std::cmp::Ordering::Equal => None,
        };
        if let Some(status) = status {
            self.send_packet(&status).await;
            self.close().await;
            return false;
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
        self.send_packet(&CNetworkSettings::new(
            compression.threshold as u16,
            0,
            false,
            0,
            0.0,
        ))
        .await;
        self.set_compression(compression).await;
        true
    }
}
