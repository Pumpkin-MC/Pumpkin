#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_config::resource_pack::BedrockPack;

const BEDROCK_PACK_CHUNK_SIZE: u32 = 1024 * 1024;

fn resolve_pack_path(pack: &BedrockPack) -> Option<std::path::PathBuf> {
    pack.path.as_ref().map_or_else(
        || {
            let candidates = [
                format!("resource_packs/{}_{}.zip", pack.uuid, pack.version),
                format!("resource_packs/{}_{}.mcpack", pack.uuid, pack.version),
                format!("resource_packs/{}.zip", pack.uuid),
                format!("resource_packs/{}.mcpack", pack.uuid),
            ];
            candidates
                .into_iter()
                .map(std::path::PathBuf::from)
                .find(|p| p.is_file())
        },
        |path| path.is_file().then(|| path.clone()),
    )
}

impl BedrockClient {
    pub async fn handle_resource_pack_response(
        &self,
        packet: SResourcePackClientResponse,
        server: &Arc<Server>,
    ) {
        // TODO: warn & ignore if the player is already spawned in

        match packet.response {
            SResourcePackClientResponse::STATUS_REFUSED => {
                debug!("Bedrock: SResourcePackResponse::STATUS_REFUSED");
                self.kick(
                    DisconnectReason::ResourcePackProblem,
                    "You must accept resource packs to join this server.".into(),
                )
                .await;
            }
            SResourcePackClientResponse::STATUS_SEND_PACKS => {
                debug!(
                    "Bedrock: SResourcePackResponse::STATUS_SEND_PACKS for {} packs: {:?}",
                    packet.pack_ids.len(),
                    packet.pack_ids
                );
                self.handle_status_send_packs(&packet.pack_ids, server)
                    .await;
            }
            SResourcePackClientResponse::STATUS_HAVE_ALL_PACKS => {
                debug!("Bedrock: SResourcePackResponse::STATUS_HAVE_ALL_PACKS");
                let br_config = &server.advanced_config.resource_pack.bedrock;
                // Convert your config packs into protocol stack entries
                let resource_packs = if br_config.enabled {
                    br_config
                        .packs
                        .iter()
                        .map(|pack| PackInstanceId {
                            pack_id: pack.uuid.to_string(),
                            version: pack.version.clone(),
                            sub_pack_name: String::new(),
                        })
                        .collect()
                } else {
                    Vec::new()
                };

                self.enqueue_client_packet(&CResourcePackStackPacket {
                    texture_pack_required: br_config.force,
                    texture_pack_list: resource_packs,
                    base_game_version: CURRENT_BEDROCK_MC_VERSION.to_string(),
                    experiments: Experiments {
                        toggles: Vec::new(),
                        experiments_ever_toggled: false,
                    },
                    include_editor_packs: false,
                })
                .await;
            }
            SResourcePackClientResponse::STATUS_COMPLETED => {
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

    async fn handle_status_send_packs(&self, pack_ids: &[String], server: &Arc<Server>) {
        let br_config = &server.advanced_config.resource_pack.bedrock;
        if !br_config.enabled {
            tracing::warn!(
                "Bedrock: Client requested packs but Bedrock resource packs are disabled"
            );
            return;
        }

        for requested_id in pack_ids {
            let matching_pack = br_config.packs.iter().find(|p| {
                p.uuid.to_string().eq_ignore_ascii_case(requested_id)
                    || format!("{}_{}", p.uuid, p.version).eq_ignore_ascii_case(requested_id)
            });

            let Some(pack) = matching_pack else {
                tracing::warn!(
                    "Bedrock: Client requested unknown resource pack: {}",
                    requested_id
                );
                if br_config.force {
                    self.kick(
                        DisconnectReason::ResourcePackProblem,
                        "Requested resource pack not found on server.".into(),
                    )
                    .await;
                    return;
                }
                continue;
            };

            if !self
                .stage_and_send_pack_info(requested_id, pack, br_config.force)
                .await
            {
                return;
            }
        }
    }

    async fn stage_and_send_pack_info(
        &self,
        requested_id: &str,
        pack: &BedrockPack,
        force: bool,
    ) -> bool {
        let Some(resolved_path) = resolve_pack_path(pack) else {
            tracing::warn!(
                "Bedrock: Pack file for {requested_id} (UUID {}) not found on disk",
                pack.uuid
            );
            if force {
                self.kick(
                    DisconnectReason::ResourcePackProblem,
                    "Required resource pack not found on server.".into(),
                )
                .await;
                return false;
            }
            return true;
        };

        let data = match tokio::fs::read(&resolved_path).await {
            Ok(bytes) => bytes,
            Err(err) => {
                tracing::error!(
                    "Bedrock: Failed to read resource pack {}: {err}",
                    resolved_path.display()
                );
                if force {
                    self.kick(
                        DisconnectReason::ResourcePackProblem,
                        "Failed to read resource pack on server.".into(),
                    )
                    .await;
                    return false;
                }
                return true;
            }
        };

        let file_hash = Sha256::digest(&data).to_vec();
        let total_size = data.len() as u64;
        let chunk_count = if total_size == 0 {
            0
        } else {
            ((total_size - 1) / u64::from(BEDROCK_PACK_CHUNK_SIZE) + 1) as u32
        };
        let pack_type = if pack.addon_pack {
            CResourcePackDataInfo::TYPE_ADDON
        } else {
            CResourcePackDataInfo::TYPE_RESOURCES
        };

        let streaming_pack = BedrockStreamingPack {
            pack_id: requested_id.to_string(),
            chunk_size: BEDROCK_PACK_CHUNK_SIZE,
            chunk_count,
            total_size,
            data: Arc::new(data),
        };

        {
            let mut lock = self
                .streaming_packs
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            lock.insert(requested_id.to_string(), streaming_pack.clone());
            lock.insert(pack.uuid.to_string(), streaming_pack.clone());
            lock.insert(format!("{}_{}", pack.uuid, pack.version), streaming_pack);
        };

        self.enqueue_client_packet(&CResourcePackDataInfo {
            pack_id: requested_id.to_string(),
            max_chunk_size: BEDROCK_PACK_CHUNK_SIZE,
            chunk_count,
            compressed_pack_size: total_size,
            file_hash,
            is_premium: false,
            pack_type,
        })
        .await;

        true
    }

    pub async fn handle_resource_pack_chunk_request(
        &self,
        packet: SResourcePackChunkRequest,
        _server: &Arc<Server>,
    ) {
        debug!(
            "Bedrock: SResourcePackChunkRequest for pack {} chunk {}",
            packet.pack_id, packet.chunk_index
        );
        let streaming_pack = {
            let guard = self
                .streaming_packs
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            guard.get(&packet.pack_id).cloned()
        };

        let Some(pack) = streaming_pack else {
            tracing::warn!(
                "Bedrock: Received chunk request for unknown or unqueued pack {}",
                packet.pack_id
            );
            return;
        };

        let offset = u64::from(packet.chunk_index) * u64::from(pack.chunk_size);
        if offset >= pack.total_size && pack.total_size > 0 {
            tracing::warn!(
                "Bedrock: Chunk request out of bounds: chunk {} offset {} size {}",
                packet.chunk_index,
                offset,
                pack.total_size
            );
            return;
        }

        let start = offset as usize;
        let end = (start + pack.chunk_size as usize).min(pack.data.len());
        let chunk_data = pack.data[start..end].to_vec();

        self.enqueue_client_packet(&CResourcePackChunkData {
            pack_id: packet.pack_id,
            chunk_index: packet.chunk_index,
            data_offset: offset,
            data: chunk_data,
        })
        .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn resolve_pack_path_custom_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let pack_file = temp_dir.path().join("test_pack.zip");
        std::fs::write(&pack_file, b"test content").unwrap();

        let pack = BedrockPack {
            uuid: Uuid::new_v4(),
            version: "1.0.0".to_string(),
            size: 12,
            download_url: String::new(),
            path: Some(pack_file.clone()),
            content_key: String::new(),
            sub_pack_name: String::new(),
            content_id: String::new(),
            has_scripts: false,
            addon_pack: false,
            rtx_enabled: false,
        };

        let resolved = resolve_pack_path(&pack);
        assert_eq!(resolved, Some(pack_file));
    }

    #[test]
    fn resolve_pack_path_missing_file_returns_none() {
        let pack = BedrockPack {
            uuid: Uuid::new_v4(),
            version: "1.0.0".to_string(),
            size: 0,
            download_url: String::new(),
            path: Some(std::path::PathBuf::from("non_existent_dir/no_pack.zip")),
            content_key: String::new(),
            sub_pack_name: String::new(),
            content_id: String::new(),
            has_scripts: false,
            addon_pack: false,
            rtx_enabled: false,
        };

        assert_eq!(resolve_pack_path(&pack), None);
    }
}
