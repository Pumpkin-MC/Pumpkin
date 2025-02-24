use std::num::NonZeroU8;

use crate::{
    entity::player::{ChatMode, Hand},
    net::{Client, PlayerConfig},
    server::Server,
};
use core::str;
use pumpkin_config::ADVANCED_CONFIG;
use pumpkin_protocol::{
    ConnectionState,
    client::config::{CFinishConfig, CRegistryData},
    codec::var_int::VarInt,
    server::config::{
        ResourcePackResponseResult, SClientInformationConfig, SConfigCookieResponse,
        SConfigResourcePack, SKnownPacks, SPluginMessage,
    },
};
use pumpkin_util::text::TextComponent;

impl Client {
    pub async fn handle_client_information_config(
        &self,
        client_information: SClientInformationConfig,
    ) {
        log::debug!("Handling client settings");
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
                view_distance: unsafe {
                    NonZeroU8::new_unchecked(client_information.view_distance as u8)
                },
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
        log::debug!("Handling plugin message");
        if plugin_message
            .channel
            .to_string()
            .starts_with("minecraft:brand")
        {
            log::debug!("got a client brand");
            match str::from_utf8(&plugin_message.data) {
                Ok(brand) => *self.brand.lock().await = Some(brand.to_string()),
                Err(e) => self.kick(TextComponent::text(e.to_string())).await,
            }
        }
    }

    pub async fn handle_resource_pack_response(&self, packet: SConfigResourcePack) {
        let resource_config = &ADVANCED_CONFIG.resource_pack;
        if resource_config.enabled {
            let expected_uuid =
                uuid::Uuid::new_v3(&uuid::Uuid::NAMESPACE_DNS, resource_config.url.as_bytes());

            if packet.uuid == expected_uuid {
                match packet.response_result() {
                    ResourcePackResponseResult::DownloadSuccess => {
                        log::trace!(
                            "Client {} successfully downloaded the resource pack",
                            self.id
                        );
                    }
                    ResourcePackResponseResult::DownloadFail => {
                        log::warn!(
                            "Client {} failed to downloaded the resource pack. Is it available on the internet?",
                            self.id
                        );
                    }
                    ResourcePackResponseResult::Downloaded => {
                        log::trace!("Client {} already has the resource pack", self.id);
                    }
                    ResourcePackResponseResult::Accepted => {
                        log::trace!("Client {} accepted the resource pack", self.id);

                        // Return here to wait for the next response update
                        return;
                    }
                    ResourcePackResponseResult::Declined => {
                        log::trace!("Client {} declined the resource pack", self.id);
                    }
                    ResourcePackResponseResult::InvalidUrl => {
                        log::warn!(
                            "Client {} reported that the resource pack url is invalid!",
                            self.id
                        );
                    }
                    ResourcePackResponseResult::ReloadFailed => {
                        log::trace!("Client {} failed to reload the resource pack", self.id);
                    }
                    ResourcePackResponseResult::Discarded => {
                        log::trace!("Client {} discarded the resource pack", self.id);
                    }
                    ResourcePackResponseResult::Unknown(result) => {
                        log::warn!(
                            "Client {} responded with a bad result: {}!",
                            self.id,
                            result
                        );
                    }
                }
            } else {
                log::warn!(
                    "Client {} returned a response for a resource pack we did not set!",
                    self.id
                );
            }
        } else {
            log::warn!(
                "Client {} returned a response for a resource pack that was not enabled!",
                self.id
            );
        }
        self.send_known_packs().await;
    }

    pub fn handle_config_cookie_response(&self, packet: SConfigCookieResponse) {
        // TODO: allow plugins to access this
        log::debug!(
            "Received cookie_response[config]: key: \"{}\", has_payload: \"{}\", payload_length: \"{}\"",
            packet.key.to_string(),
            packet.has_payload,
            packet.payload_length.unwrap_or(VarInt::from(0)).0
        );
    }

    pub async fn handle_known_packs(&self, server: &Server, _config_acknowledged: SKnownPacks) {
        log::debug!("Handling known packs");
        for registry in &server.cached_registry {
            self.send_packet(&CRegistryData::new(
                &registry.registry_id,
                &registry.registry_entries,
            ))
            .await;
        }

        // We are done with configuring
        log::debug!("finished config");
        self.send_packet(&CFinishConfig).await;
    }

    pub async fn handle_config_acknowledged(&self) {
        log::debug!("Handling config acknowledge");
        self.connection_state.store(ConnectionState::Play);

        if let Some(reason) = self.can_not_join().await {
            self.kick(reason).await;
            return;
        }

        self.make_player
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}
