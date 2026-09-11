#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub async fn handle_resource_pack_response(
        &self,
        server: &Server,
        packet: SConfigResourcePack,
    ) {
        let resource_config = &server.advanced_config.resource_pack.java;
        if resource_config.enabled {
            let expected_uuid =
                uuid::Uuid::new_v3(&uuid::Uuid::NAMESPACE_DNS, resource_config.url.as_bytes());

            if packet.uuid == expected_uuid {
                match packet.response_result() {
                    ResourcePackResponseResult::DownloadSuccess => {
                        trace!(
                            "Client {} successfully downloaded the resource pack",
                            self.id
                        );
                    }
                    ResourcePackResponseResult::DownloadFail => {
                        warn!(
                            "Client {} failed to downloaded the resource pack. Is it available on the internet?",
                            self.id
                        );
                    }
                    ResourcePackResponseResult::Downloaded => {
                        trace!("Client {} already has the resource pack", self.id);
                    }
                    ResourcePackResponseResult::Accepted => {
                        trace!("Client {} accepted the resource pack", self.id);

                        // Return here to wait for the next response update
                        return;
                    }
                    ResourcePackResponseResult::Declined => {
                        trace!("Client {} declined the resource pack", self.id);
                    }
                    ResourcePackResponseResult::InvalidUrl => {
                        warn!(
                            "Client {} reported that the resource pack URL is invalid!",
                            self.id
                        );
                    }
                    ResourcePackResponseResult::ReloadFailed => {
                        trace!("Client {} failed to reload the resource pack", self.id);
                    }
                    ResourcePackResponseResult::Discarded => {
                        trace!("Client {} discarded the resource pack", self.id);
                    }
                    ResourcePackResponseResult::Unknown(result) => {
                        warn!(
                            "Client {} responded with a bad result: {}!",
                            self.id, result
                        );
                    }
                }
            } else {
                warn!(
                    "Client {} returned a response for a resource pack we did not set!",
                    self.id
                );
            }
        } else {
            warn!(
                "Client {} returned a response for a resource pack that was not enabled!",
                self.id
            );
        }
        self.send_known_packs().await;
    }

    pub async fn send_known_packs(&self) {
        let version_str = self.version.load().to_string();
        self.send_packet(&CKnownPacks::new(&[KnownPack {
            namespace: "minecraft",
            id: "core",
            version: &version_str,
        }]))
        .await;
    }
}
