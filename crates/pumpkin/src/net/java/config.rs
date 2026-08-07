use std::{num::NonZeroU8, sync::Arc, sync::atomic::Ordering};

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
    ConnectionState, KnownPack,
    java::{
        client::config::{CFeatureFlags, CFinishConfig, CKnownPacks, CRegistryData, CUpdateTags},
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
        client_information: SClientInformationConfig<'_>,
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
            self.config.store(Arc::new(PlayerConfig {
                locale: client_information.locale.to_string(),
                // client_information.view_distance was checked above to be > 0 so compiler should optimize this out.
                view_distance: NonZeroU8::new(client_information.view_distance as u8).unwrap(),
                chat_mode,
                chat_colors: client_information.chat_colors,
                skin_parts: client_information.skin_parts,
                main_hand,
                text_filtering: client_information.text_filtering,
                server_listing: client_information.server_listing,
            }));
        } else {
            self.kick(TextComponent::text("Invalid hand or chat type"))
                .await;
        }
    }

    pub async fn handle_plugin_message(&self, plugin_message: SPluginMessage<'_>) {
        debug!("Handling plugin message");
        if plugin_message.channel.starts_with(BRAND_CHANNEL_PREFIX) {
            debug!("Got a client brand");
            match str::from_utf8(plugin_message.data) {
                Ok(brand) => self.brand.store(Arc::new(Some(brand.to_string()))),
                Err(e) => self.kick(TextComponent::text(e.to_string())).await,
            }
        }
    }

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
        self.send_packet_now(&CKnownPacks::new(&[KnownPack {
            namespace: "minecraft",
            id: "core",
            version: &version_str,
        }]))
        .await;
    }

    pub fn handle_config_cookie_response(&self, packet: &SConfigCookieResponse<'_>) {
        // TODO: allow plugins to access this
        debug!(
            "Received cookie_response[config]: key: \"{}\", has_payload: \"{}\", payload_length: \"{:?}\"",
            packet.key,
            packet.has_payload,
            packet.payload.as_ref().map(|p| p.len()),
        );
    }

    pub async fn handle_known_packs(
        &self,
        _config_acknowledged: SKnownPacks<'_>,
        server: &Server,
    ) -> Option<PacketHandlerResult> {
        debug!("Handling known packs");
        // let mut tags_to_send = Vec::new();
        let version = self.version.load();
        if version >= JavaMinecraftVersion::V_1_20_2 {
            self.send_packet_now(&CFeatureFlags::new(&["minecraft:vanilla".to_string()]))
                .await;
            let registry = Registry::get_synced(version);
            let mut sent_dimension_type = false;
            for reg in &registry {
                if reg.registry_id == "minecraft:dimension_type" {
                    sent_dimension_type = true;
                }
                self.send_packet_now(&CRegistryData::new(&reg.registry_id, &reg.registry_entries))
                    .await;
            }
            if !sent_dimension_type {
                let dims = [
                    &pumpkin_data::dimension::Dimension::OVERWORLD,
                    &pumpkin_data::dimension::Dimension::OVERWORLD_CAVES,
                    &pumpkin_data::dimension::Dimension::THE_END,
                    &pumpkin_data::dimension::Dimension::THE_NETHER,
                ];
                let dim_entries: Vec<pumpkin_data::registry::RegistryEntryData> = dims
                    .iter()
                    .map(|dim| pumpkin_data::registry::RegistryEntryData {
                        entry_id: dim.minecraft_name.to_string(),
                        data: Some(build_dimension_nbt(dim).into_boxed_slice()),
                    })
                    .collect();
                self.send_packet_now(&CRegistryData::new(
                    &"minecraft:dimension_type".to_string(),
                    &dim_entries,
                ))
                .await;
            }
        }
        let all_keys = [
            pumpkin_data::tag::RegistryKey::BannerPattern,
            pumpkin_data::tag::RegistryKey::Block,
            pumpkin_data::tag::RegistryKey::CatVariant,
            pumpkin_data::tag::RegistryKey::DamageType,
            pumpkin_data::tag::RegistryKey::Dialog,
            pumpkin_data::tag::RegistryKey::DimensionType,
            pumpkin_data::tag::RegistryKey::Enchantment,
            pumpkin_data::tag::RegistryKey::EntityType,
            pumpkin_data::tag::RegistryKey::Fluid,
            pumpkin_data::tag::RegistryKey::GameEvent,
            pumpkin_data::tag::RegistryKey::Instrument,
            pumpkin_data::tag::RegistryKey::Item,
            pumpkin_data::tag::RegistryKey::PaintingVariant,
            pumpkin_data::tag::RegistryKey::PointOfInterestType,
            pumpkin_data::tag::RegistryKey::Potion,
            pumpkin_data::tag::RegistryKey::Timeline,
            pumpkin_data::tag::RegistryKey::WorldgenBiome,
        ];

        let mut tags = Vec::new();
        for key in all_keys {
            if pumpkin_data::tag::get_registry_key_tags(version, key)
                .is_some_and(|map| !map.is_empty())
            {
                tags.push(key);
            }
        }
        self.send_packet_now(&CUpdateTags::new(&tags)).await;

        // We are done with configuring
        self.send_packet_now(&CFinishConfig).await;

        if version < JavaMinecraftVersion::V_1_20_2 {
            return Some(self.handle_config_acknowledged(server).await);
        }

        debug!("Finished config");
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

    pub async fn handle_config_acknowledged(&self, server: &Server) -> PacketHandlerResult {
        debug!("Handling config acknowledgement");
        self.connection_state.store(ConnectionState::Play);

        let profile = self.gameprofile.clone();
        let address = self.address;

        if let Some(reason) = can_not_join(&profile, &address, server).await {
            self.kick(reason).await;
            return PacketHandlerResult::Stop;
        }

        let config = self.config.load();
        PacketHandlerResult::ReadyToPlay(profile, (**config).clone())
    }
}

fn build_dimension_nbt(dim: &pumpkin_data::dimension::Dimension) -> Vec<u8> {
    let mut compound = pumpkin_nbt::compound::NbtCompound::new();
    compound.put_float("ambient_light", dim.ambient_light);
    compound.put_int("height", dim.height);
    compound.put_int("logical_height", dim.logical_height);
    compound.put_int("min_y", dim.min_y);
    compound.put_string("infiniburn", dim.infiniburn.to_string());
    compound.put_int(
        "monster_spawn_block_light_limit",
        dim.monster_spawn_block_light_limit as i32,
    );
    compound.put_double("coordinate_scale", dim.coordinate_scale);
    compound.put_byte("has_skylight", i8::from(dim.has_skylight));
    compound.put_byte("has_ceiling", i8::from(dim.has_ceiling));
    compound.put_byte("ultrawarm", i8::from(dim.id == 3));
    compound.put_byte("natural", i8::from(dim.id == 0 || dim.id == 1));
    compound.put_byte("piglin_safe", i8::from(dim.id == 3));
    compound.put_byte("respawn_anchor_works", i8::from(dim.id == 3));
    compound.put_byte("bed_works", i8::from(dim.id == 0 || dim.id == 1));
    compound.put_byte("has_raids", i8::from(dim.id == 0 || dim.id == 1));
    compound.put_string("effects", dim.minecraft_name.to_string());

    let mut monster_spawn = pumpkin_nbt::compound::NbtCompound::new();
    monster_spawn.put_string("type", "minecraft:uniform".to_string());
    let mut value = pumpkin_nbt::compound::NbtCompound::new();
    value.put_int("min_inclusive", 0);
    value.put_int("max_inclusive", 7);
    monster_spawn.put_compound("value", value);
    compound.put_compound("monster_spawn_light_level", monster_spawn);

    pumpkin_nbt::Nbt::from(compound).write().to_vec()
}
