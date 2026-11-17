#[allow(clippy::wildcard_imports)]
use super::*;
use crate::net::bedrock::BedrockVersions;
use std::sync::atomic::Ordering;

const INCOMPATIBLE_PROTOCOL_FLUSH_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(1);

/// Bedrock 1.20.60 (protocol 649) added a 1-byte compression method prefix after `0xFE`.
const PREFIXED_COMPRESSION_MIN_PROTOCOL: u32 = 649;

fn incompatible_protocol_status(
    client_network_version: i32,
    versions: &BedrockVersions,
) -> Option<CPlayStatus> {
    if let Ok(protocol) = u32::try_from(client_network_version)
        && versions.admits_protocol(protocol)
    {
        return None;
    }
    match client_network_version.cmp(&(CURRENT_BEDROCK_MC_PROTOCOL as i32)) {
        std::cmp::Ordering::Less => Some(CPlayStatus::OutdatedClient),
        std::cmp::Ordering::Greater => Some(CPlayStatus::OutdatedServer),
        std::cmp::Ordering::Equal => None,
    }
}

impl BedrockClient {
    async fn reject_incompatible_protocol(&self, status: CPlayStatus) {
        self.send_packet(&status).await;
        if let Err(error) = self
            .session
            .flush_reliable(INCOMPATIBLE_PROTOCOL_FLUSH_TIMEOUT)
            .await
        {
            debug!(
                address = %self.address,
                %error,
                "Failed to flush Bedrock version rejection"
            );
        }
        self.close().await;
    }

    /// Admits a pre-1.19.30 client that sends `SLogin` directly without `SRequestNetworkSettings`.
    pub async fn admit_legacy_login_protocol(
        &self,
        client_network_version: i32,
        server: &Server,
    ) -> bool {
        if let Some(status) = incompatible_protocol_status(
            client_network_version,
            &server.plugin_manager.bedrock_versions,
        ) {
            self.reject_incompatible_protocol(status).await;
            return false;
        }
        let protocol = client_network_version as u32;
        self.protocol_version.store(protocol, Ordering::Relaxed);
        self.version
            .store(BedrockMinecraftVersion::from_protocol(protocol));
        self.set_prefixed_compression(protocol >= PREFIXED_COMPRESSION_MIN_PROTOCOL)
            .await;
        self.set_compression(
            server
                .advanced_config
                .networking
                .bedrock
                .compression
                .info
                .clone(),
        )
        .await;
        true
    }

    pub async fn handle_request_network_settings(
        &self,
        packet: SRequestNetworkSettings,
        server: &Server,
    ) -> bool {
        if let Some(status) = incompatible_protocol_status(
            packet.client_network_version,
            &server.plugin_manager.bedrock_versions,
        ) {
            self.reject_incompatible_protocol(status).await;
            return false;
        }

        let protocol = packet.client_network_version as u32;
        self.protocol_version.store(protocol, Ordering::Relaxed);
        self.version
            .store(BedrockMinecraftVersion::from_protocol(protocol));
        self.set_prefixed_compression(protocol >= PREFIXED_COMPRESSION_MIN_PROTOCOL)
            .await;

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
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_current_and_registered_bedrock_protocols() {
        let versions = BedrockVersions::new();
        assert!(matches!(
            incompatible_protocol_status(2224, &versions),
            Some(CPlayStatus::OutdatedClient)
        ));
        assert!(incompatible_protocol_status(2225, &versions).is_none());
        assert!(matches!(
            incompatible_protocol_status(2226, &versions),
            Some(CPlayStatus::OutdatedServer)
        ));

        versions.register("multiversion", [671, 2193]);
        assert!(incompatible_protocol_status(671, &versions).is_none());
        assert!(incompatible_protocol_status(2193, &versions).is_none());
        assert!(matches!(
            incompatible_protocol_status(-1, &versions),
            Some(CPlayStatus::OutdatedClient)
        ));
    }
}
