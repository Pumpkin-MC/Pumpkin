#[allow(clippy::wildcard_imports)]
use super::*;
use bytes::Bytes;
use std::collections::HashMap;

impl JavaClient {
    pub async fn handle_known_packs(&self, server: &Server) -> Option<PacketHandlerResult> {
        debug!("Handling known packs");

        let version = self.version.load();

        if version.supports_configuration_state() {
            if version < JavaMinecraftVersion::V_1_20_5 {
                let features = server.get_enabled_features();
                self.send_packet(&CFeatureFlags::new(&features)).await;
            }

            let test_instance_entries =
                server.datapack_manager.get_test_instance_registry_entries();
            let custom_damage_type_map = server.datapack_manager.get_damage_type_registry_map();

            let packets = tokio::task::spawn_blocking(move || {
                Self::build_synced_registry_packets(
                    version,
                    &test_instance_entries,
                    &custom_damage_type_map,
                )
            })
            .await
            .unwrap_or_default();

            for packet_data in packets {
                self.send_packet_now(packet_data).await;
            }
        }

        // We are done with configuring
        self.send_packet(&CFinishConfig).await;

        if !version.supports_configuration_state() {
            return Some(self.handle_config_acknowledged(server).await);
        }

        debug!("Finished config");
        None
    }

    fn build_synced_registry_packets<S: std::hash::BuildHasher>(
        version: JavaMinecraftVersion,
        test_instance_entries: &[pumpkin_data::registry::RegistryEntryData],
        custom_damage_type_map: &HashMap<
            String,
            crate::data::datapack::damage_type_loader::DamageTypeEntry,
            S,
        >,
    ) -> Vec<Bytes> {
        let registry = Registry::get_synced(version);
        let mut packets = Vec::new();
        let mut sent_dimension_type = false;
        let mut sent_test_instance = false;
        let mut sent_damage_type = false;

        for reg in &registry {
            if reg.registry_id == "minecraft:dimension_type" {
                sent_dimension_type = true;
            }

            if reg.registry_id == "minecraft:test_instance" {
                sent_test_instance = true;

                let packet = CRegistryData::new(&reg.registry_id, test_instance_entries);

                if let Ok(data) = Self::serialize_packet_for_version(&packet, version) {
                    packets.push(data);
                }

                continue;
            }

            if reg.registry_id == "minecraft:damage_type" {
                sent_damage_type = true;

                let merged_damage_types = crate::data::datapack::merge_damage_type_entries(
                    &reg.registry_entries,
                    custom_damage_type_map,
                );

                let packet = CRegistryData::new(&reg.registry_id, &merged_damage_types);

                if let Ok(data) = Self::serialize_packet_for_version(&packet, version) {
                    packets.push(data);
                }

                continue;
            }

            let packet = CRegistryData::new(&reg.registry_id, &reg.registry_entries);

            if let Ok(data) = Self::serialize_packet_for_version(&packet, version) {
                packets.push(data);
            }
        }

        // The generated synced-registry table can lag behind the current protocol.
        // ResourceSelectorArgument validates /test names against this registry on the
        // client, so always provide it even when the static table omitted it.
        if !sent_test_instance {
            let test_instance = "minecraft:test_instance".to_string();
            let packet = CRegistryData::new(&test_instance, test_instance_entries);

            if let Ok(data) = Self::serialize_packet_for_version(&packet, version) {
                packets.push(data);
            }
        }

        if !sent_damage_type && !custom_damage_type_map.is_empty() {
            let damage_type = "minecraft:damage_type".to_string();
            let merged_damage_types =
                crate::data::datapack::merge_damage_type_entries(&[], custom_damage_type_map);
            let packet = CRegistryData::new(&damage_type, &merged_damage_types);

            if let Ok(data) = Self::serialize_packet_for_version(&packet, version) {
                packets.push(data);
            }
        }

        if !sent_dimension_type {
            let dim_entries = get_fallback_dimension_type_entries();
            let dim_type = "minecraft:dimension_type".to_string();
            let packet = CRegistryData::new(&dim_type, &dim_entries);

            if let Ok(data) = Self::serialize_packet_for_version(&packet, version) {
                packets.push(data);
            }
        }

        let tags = get_network_tag_keys(version);
        let packet = CUpdateTags::new(&tags);

        if let Ok(data) = Self::serialize_packet_for_version(&packet, version) {
            packets.push(data);
        }

        packets
    }
}

fn get_fallback_dimension_type_entries() -> Vec<pumpkin_data::registry::RegistryEntryData> {
    let dims = [
        &pumpkin_data::dimension::Dimension::OVERWORLD,
        &pumpkin_data::dimension::Dimension::OVERWORLD_CAVES,
        &pumpkin_data::dimension::Dimension::THE_END,
        &pumpkin_data::dimension::Dimension::THE_NETHER,
    ];

    dims.iter()
        .map(|dim| pumpkin_data::registry::RegistryEntryData {
            entry_id: dim.minecraft_name.to_string(),
            data: Some(build_dimension_nbt(dim).into_boxed_slice()),
        })
        .collect()
}

fn get_network_tag_keys(version: JavaMinecraftVersion) -> Vec<pumpkin_data::tag::RegistryKey> {
    let mut tags = Vec::new();

    for &key in pumpkin_data::tag::RegistryKey::NETWORK_KEYS {
        if pumpkin_data::tag::get_registry_key_tags(version, key).is_some_and(|map| !map.is_empty())
        {
            tags.push(key);
        }
    }

    tags
}
