use std::{
    num::NonZeroU8,
    sync::{Arc, atomic::Ordering},
};

use pumpkin_util::translation::{localized_log, localized_log_format, localized_text};

use crate::{
    entity::player::ChatMode,
    net::{
        PlayerConfig, can_not_join,
        java::{JavaClient, PacketHandlerResult},
    },
    server::Server,
};
use core::str;
use pumpkin_data::{registry::Registry, translation};
use pumpkin_protocol::{
    ConnectionState,
    java::{
        client::config::{CFinishConfig, CRegistryData, CUpdateTags, RegistryEntry},
        server::config::{
            ResourcePackResponseResult, SClientInformationConfig, SConfigCookieResponse,
            SConfigResourcePack, SKeepAlive, SKnownPacks, SPluginMessage,
        },
    },
};
use pumpkin_util::{Hand, text::TextComponent, version::JavaMinecraftVersion};
use tracing::{debug, trace, warn};

const BRAND_CHANNEL_PREFIX: &str = "minecraft:brand";

impl JavaClient {
    pub async fn handle_client_information_config(
        &self,
        client_information: SClientInformationConfig,
    ) {
        debug!("{}", localized_log("server.log.java_client_settings"));
        if client_information.view_distance <= 0 {
            self.kick(localized_text(
                "client.disconnect.invalid_view_distance",
                [],
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
            self.kick(localized_text(
                "client.disconnect.invalid_hand_or_chat_type",
                [],
            ))
            .await;
        }
    }

    pub async fn handle_plugin_message(&self, plugin_message: SPluginMessage) {
        debug!("{}", localized_log("server.log.java_plugin_message"));
        if plugin_message.channel.starts_with(BRAND_CHANNEL_PREFIX) {
            debug!("{}", localized_log("server.log.java_client_brand"));
            match str::from_utf8(&plugin_message.data) {
                Ok(brand) => *self.brand.lock().await = Some(brand.to_string()),
                Err(e) => self.kick(TextComponent::text(e.to_string())).await,
            }
        }
    }

    #[allow(clippy::too_many_lines)]
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
                            "{}",
                            localized_log_format(
                                "server.log.resource_pack_download_success",
                                &[self.id.to_string()]
                            )
                        );
                    }
                    ResourcePackResponseResult::DownloadFail => {
                        warn!(
                            "{}",
                            localized_log_format(
                                "server.log.resource_pack_download_failed",
                                &[self.id.to_string()]
                            )
                        );
                    }
                    ResourcePackResponseResult::Downloaded => {
                        trace!(
                            "{}",
                            localized_log_format(
                                "server.log.resource_pack_already_downloaded",
                                &[self.id.to_string()]
                            )
                        );
                    }
                    ResourcePackResponseResult::Accepted => {
                        trace!(
                            "{}",
                            localized_log_format(
                                "server.log.resource_pack_accepted",
                                &[self.id.to_string()]
                            )
                        );

                        // Return here to wait for the next response update
                        return;
                    }
                    ResourcePackResponseResult::Declined => {
                        trace!(
                            "{}",
                            localized_log_format(
                                "server.log.resource_pack_declined",
                                &[self.id.to_string()]
                            )
                        );
                    }
                    ResourcePackResponseResult::InvalidUrl => {
                        warn!(
                            "{}",
                            localized_log_format(
                                "server.log.resource_pack_invalid_url",
                                &[self.id.to_string()]
                            )
                        );
                    }
                    ResourcePackResponseResult::ReloadFailed => {
                        trace!(
                            "{}",
                            localized_log_format(
                                "server.log.resource_pack_reload_failed",
                                &[self.id.to_string()]
                            )
                        );
                    }
                    ResourcePackResponseResult::Discarded => {
                        trace!(
                            "{}",
                            localized_log_format(
                                "server.log.resource_pack_discarded",
                                &[self.id.to_string()]
                            )
                        );
                    }
                    ResourcePackResponseResult::Unknown(result) => {
                        warn!(
                            "{}",
                            localized_log_format(
                                "server.log.resource_pack_bad_result",
                                &[self.id.to_string(), result.to_string()]
                            )
                        );
                    }
                }
            } else {
                warn!(
                    "{}",
                    localized_log_format(
                        "server.log.resource_pack_unexpected_response",
                        &[self.id.to_string()]
                    )
                );
            }
        } else {
            warn!(
                "{}",
                localized_log_format(
                    "server.log.resource_pack_response_when_disabled",
                    &[self.id.to_string()]
                )
            );
        }
        self.send_known_packs().await;
    }

    pub fn handle_config_cookie_response(&self, packet: &SConfigCookieResponse) {
        // TODO: allow plugins to access this
        debug!(
            "{}",
            localized_log_format(
                "server.log.config_cookie_response",
                &[
                    packet.key.to_string(),
                    packet.has_payload.to_string(),
                    format!("{:?}", packet.payload.as_ref().map(|p| p.len()))
                ]
            )
        );
    }

    pub async fn handle_known_packs(
        &self,
        _config_acknowledged: SKnownPacks,
        server: &Arc<Server>,
    ) -> Option<PacketHandlerResult> {
        debug!("{}", localized_log("server.log.java_known_packs"));
        // let mut tags_to_send = Vec::new();
        let version = self.version.load();
        let registry = Registry::get_synced(version);
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

        // optionally include timeline/dimension_type if there are any tags to send
        if version.protocol_version() >= JavaMinecraftVersion::V_1_21_11.protocol_version()
            && let Some(map) = pumpkin_data::tag::get_registry_key_tags(
                version,
                pumpkin_data::tag::RegistryKey::Timeline,
            )
            && !map.is_empty()
        {
            tags.push(pumpkin_data::tag::RegistryKey::Timeline);
        }
        if let Some(map) = pumpkin_data::tag::get_registry_key_tags(
            version,
            pumpkin_data::tag::RegistryKey::DimensionType,
        ) && !map.is_empty()
        {
            tags.push(pumpkin_data::tag::RegistryKey::DimensionType);
        }
        if let Some(map) = pumpkin_data::tag::get_registry_key_tags(
            version,
            pumpkin_data::tag::RegistryKey::DamageType,
        ) && !map.is_empty()
        {
            tags.push(pumpkin_data::tag::RegistryKey::DamageType);
        }
        if let Some(map) = pumpkin_data::tag::get_registry_key_tags(
            version,
            pumpkin_data::tag::RegistryKey::BannerPattern,
        ) && !map.is_empty()
        {
            tags.push(pumpkin_data::tag::RegistryKey::BannerPattern);
        }
        self.send_packet_now(&CUpdateTags::new(&tags)).await;

        // We are done with configuring
        self.send_packet_now(&CFinishConfig).await;

        if version < JavaMinecraftVersion::V_1_20_2 {
            return Some(self.handle_config_acknowledged(server).await);
        }

        debug!("{}", localized_log("server.log.java_config_finished"));
        None
    }

    pub async fn handle_config_keep_alive(&self, keep_alive: SKeepAlive) {
        if self.wait_for_keep_alive.load(Ordering::Relaxed)
            && keep_alive.keep_alive_id == self.keep_alive_id.load()
        {
            self.wait_for_keep_alive.store(false, Ordering::Relaxed);
        } else {
            self.kick(TextComponent::translate(
                translation::java::DISCONNECT_TIMEOUT,
                [],
            ))
            .await;
        }
    }

    pub async fn handle_config_acknowledged(&self, server: &Arc<Server>) -> PacketHandlerResult {
        debug!(
            "{}",
            localized_log("server.log.java_config_acknowledgement")
        );
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
