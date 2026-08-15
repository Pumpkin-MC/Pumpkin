#[allow(clippy::wildcard_imports)]
use super::*;

impl BedrockClient {
    pub async fn handle_resource_pack_response(
        &self,
        packet: SResourcePackResponse,
        server: &Arc<Server>,
    ) {
        // TODO: warn & ignore if the player is already spawned in

        match packet.response {
            SResourcePackResponse::STATUS_REFUSED => {
                debug!("Bedrock: SResourcePackResponse::STATUS_REFUSED");
                self.kick(
                    DisconnectReason::ResourcePackProblem,
                    "You must accept resource packs to join this server.".into(),
                )
                .await;
            }
            SResourcePackResponse::STATUS_SEND_PACKS => {
                debug!("Bedrock: SResourcePackResponse::STATUS_SEND_PACKS");
                // TODO: send packs
            }
            SResourcePackResponse::STATUS_HAVE_ALL_PACKS => {
                debug!("Bedrock: SResourcePackResponse::STATUS_HAVE_ALL_PACKS");
                let br_config = &server.advanced_config.resource_pack.bedrock;
                // Convert your config packs into protocol stack entries
                let resource_packs = if br_config.enabled {
                    br_config
                        .packs
                        .iter()
                        .map(|pack| ResourcePackStackEntry {
                            uuid: pack.uuid.to_string(),
                            version: pack.version.clone(),
                            sub_pack_name: String::new(),
                        })
                        .collect()
                } else {
                    Vec::new()
                };

                self.enqueue_packet_internal(&CResourcePackStackPacket::new(
                    br_config.force,
                    resource_packs,
                    CURRENT_BEDROCK_MC_VERSION.to_string(),
                    Experiments {
                        names_size: 0,
                        experiments_ever_toggled: false,
                    },
                    false,
                ))
                .await;
            }
            SResourcePackResponse::STATUS_COMPLETED => {
                debug!("Bedrock: SResourcePackResponse::STATUS_COMPLETED");
                let player = self.player.load_full();
                if let Some(player) = player.as_ref() {
                    player
                        .world()
                        .spawn_bedrock_player(&server.basic_config, player.clone(), server)
                        .await;
                } else {
                    tracing::error!(
                        "Got SResourcePackResponse::STATUS_COMPLETED before authentication was completed."
                    );
                    self.kick(DisconnectReason::Disconnected, String::new())
                        .await;
                }
            }
            _ => {
                tracing::error!("Bedrock: SResourcePackResponse bad response type");
                self.kick(DisconnectReason::Disconnected, String::new())
                    .await;
            }
        }
    }
}
