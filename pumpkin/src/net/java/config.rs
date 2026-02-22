use std::{num::NonZeroU8, sync::Arc};

use crate::{
    entity::player::ChatMode,
    net::{
        PlayerConfig, can_not_join,
        java::{JavaClient, PacketHandlerResult},
    },
    plugin::player::resource_pack_status::PlayerResourcePackStatusEvent,
    server::Server,
};
use core::str;
use pumpkin_data::registry::Registry;
use pumpkin_protocol::{
    ConnectionState,
    java::{
        client::config::{CFinishConfig, CRegistryData, CUpdateTags, RegistryEntry},
        server::config::{
            ResourcePackResponseResult, SClientInformationConfig, SConfigCookieResponse,
            SConfigResourcePack, SKnownPacks, SPluginMessage,
        },
    },
};
use pumpkin_util::{Hand, text::TextComponent, version::MinecraftVersion};
use tracing::{debug, trace, warn};

const BRAND_CHANNEL_PREFIX: &str = "minecraft:brand";

impl JavaClient {
    pub async fn handle_client_information_config(
        &self,
        client_information: SClientInformationConfig,
    ) {
        debug!("Handling client settings");
        if client_information.view_distance <= 0 {
            self.kick(TextComponent::text(
                "Cannot have zero or negative view distance!",
            ))
            .await;
            return;
        }

        if let (Ok(main_hand), Ok(chat_mode)) = (
            Hand::try_from(client_information.main_hand.0),
            ChatMode::try_from(client_information.chat_mode.0),
        ) {
            *self.config.lock().await = Some(PlayerConfig {
                locale: client_information.locale,
                // client_information.view_distance was checked above to be > 0 so compiler should optimize this out.
                view_distance: NonZeroU8::new(client_information.view_distance as u8).unwrap(),
                chat_mode,
                chat_colors: client_information.chat_colors,
                skin_parts: client_information.skin_parts,
                main_hand,
                text_filtering: client_information.text_filtering,
                server_listing: client_information.server_listing,
            });
        } else {
            self.kick(TextComponent::text("Invalid hand or chat type"))
                .await;
        }
    }

    pub async fn handle_plugin_message(&self, plugin_message: SPluginMessage) {
        debug!("Handling plugin message");
        if plugin_message.channel.starts_with(BRAND_CHANNEL_PREFIX) {
            debug!("Got a client brand");
            match str::from_utf8(&plugin_message.data) {
                Ok(brand) => *self.brand.lock().await = Some(brand.to_string()),
                Err(e) => self.kick(TextComponent::text(e.to_string())).await,
            }
        }
    }

    pub async fn handle_resource_pack_response(
        &self,
        server: &Server,
        packet: SConfigResourcePack,
    ) {
        let resource_config = &server.advanced_config.resource_pack;
        if resource_config.enabled {
            let expected_uuid =
                uuid::Uuid::new_v3(&uuid::Uuid::NAMESPACE_DNS, resource_config.url.as_bytes());

            if packet.uuid == expected_uuid {
                let status = match packet.response_result() {
                    ResourcePackResponseResult::DownloadSuccess => {
                        trace!(
                            "Client {} successfully downloaded the resource pack",
                            self.id
                        );
                        "SUCCESSFULLY_LOADED"
                    }
                    ResourcePackResponseResult::DownloadFail => {
                        warn!(
                            "Client {} failed to downloaded the resource pack. Is it available on the internet?",
                            self.id
                        );
                        "FAILED_DOWNLOAD"
                    }
                    ResourcePackResponseResult::Downloaded => {
                        trace!("Client {} already has the resource pack", self.id);
                        "DOWNLOADED"
                    }
                    ResourcePackResponseResult::Accepted => {
                        trace!("Client {} accepted the resource pack", self.id);

                        // Return here to wait for the next response update
                        return;
                    }
                    ResourcePackResponseResult::Declined => {
                        trace!("Client {} declined the resource pack", self.id);
                        "DECLINED"
                    }
                    ResourcePackResponseResult::InvalidUrl => {
                        warn!(
                            "Client {} reported that the resource pack URL is invalid!",
                            self.id
                        );
                        "INVALID_URL"
                    }
                    ResourcePackResponseResult::ReloadFailed => {
                        trace!("Client {} failed to reload the resource pack", self.id);
                        "FAILED_RELOAD"
                    }
                    ResourcePackResponseResult::Discarded => {
                        trace!("Client {} discarded the resource pack", self.id);
                        "DISCARDED"
                    }
                    ResourcePackResponseResult::Unknown(result) => {
                        warn!(
                            "Client {} responded with a bad result: {}!",
                            self.id, result
                        );
                        "FAILED_DOWNLOAD"
                    }
                };

                let profile = self.gameprofile.lock().await.clone();
                if let Some(profile) = profile
                    && let Some(player) = server.get_player_by_uuid(profile.id)
                {
                    let event = PlayerResourcePackStatusEvent::new(
                        player,
                        packet.uuid,
                        resource_config.sha1.clone(),
                        status.to_string(),
                    );
                    server.plugin_manager.fire(event).await;
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

    pub fn handle_config_cookie_response(&self, packet: &SConfigCookieResponse) {
        // TODO: allow plugins to access this
        debug!(
            "Received cookie_response[config]: key: \"{}\", has_payload: \"{}\", payload_length: \"{:?}\"",
            packet.key,
            packet.has_payload,
            packet.payload.as_ref().map(|p| p.len()),
        );
    }

    pub async fn handle_known_packs(&self, _config_acknowledged: SKnownPacks) {
        debug!("Handling known packs");
        // let mut tags_to_send = Vec::new();
        let registry = Registry::get_synced(self.version.load());
        for registry in registry {
            let entries: Vec<RegistryEntry> = registry
                .registry_entries
                .iter()
                .map(|r| RegistryEntry::new(r.entry_id.clone(), r.data.clone()))
                .collect();
            self.send_packet_now(&CRegistryData::new(&registry.registry_id, &entries))
                .await;
            // if let Some(tag) = RegistryKey::from_string(&registry.registry_id.path)
            //     && pumpkin_data::tag::get_registry_key_tags(self.version.load(), tag).is_some()
            // {
            //     tags_to_send.push(tag);
            // }
        }
        //self.send_packet_now(&CUpdateTags::new(&tags_to_send)).await;
        let mut tags = vec![
            pumpkin_data::tag::RegistryKey::Block,
            pumpkin_data::tag::RegistryKey::Fluid,
            pumpkin_data::tag::RegistryKey::Enchantment,
            pumpkin_data::tag::RegistryKey::WorldgenBiome,
            pumpkin_data::tag::RegistryKey::Item,
            pumpkin_data::tag::RegistryKey::EntityType,
            pumpkin_data::tag::RegistryKey::Dialog,
        ];
        if self.version.load().protocol_version() >= MinecraftVersion::V_1_21_11.protocol_version()
        {
            tags.push(pumpkin_data::tag::RegistryKey::Timeline);
        }
        self.send_packet_now(&CUpdateTags::new(&tags)).await;

        // We are done with configuring
        debug!("Finished config");
        self.send_packet_now(&CFinishConfig).await;
    }

    pub async fn handle_config_acknowledged(&self, server: &Arc<Server>) -> PacketHandlerResult {
        debug!("Handling config acknowledgement");
        self.connection_state.store(ConnectionState::Play);

        let profile = self.gameprofile.lock().await.clone();
        let profile = profile.unwrap();
        let address = self.address.lock().await;

        if let Some(reason) = can_not_join(&profile, &address, server).await {
            self.kick(reason).await;
            return PacketHandlerResult::Stop;
        }

        let config = self.config.lock().await;
        PacketHandlerResult::ReadyToPlay(profile, config.clone().unwrap_or_default())
    }
}
