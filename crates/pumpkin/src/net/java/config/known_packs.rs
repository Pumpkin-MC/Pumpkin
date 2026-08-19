#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub async fn handle_known_packs(
        &self,
        _config_acknowledged: SKnownPacks<'_>,
        server: &Server,
    ) -> Option<PacketHandlerResult> {
        debug!("Handling known packs");
        // let mut tags_to_send = Vec::new();
        let version = self.version.load();
        if version >= JavaMinecraftVersion::V_1_20_2 {
            self.send_packet(&CFeatureFlags::new(&["minecraft:vanilla".to_string()]))
                .await;
            let registry = Registry::get_synced(version);
            let mut sent_dimension_type = false;
            for reg in &registry {
                if reg.registry_id == "minecraft:dimension_type" {
                    sent_dimension_type = true;
                }
                self.send_packet(&CRegistryData::new(&reg.registry_id, &reg.registry_entries))
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
                self.send_packet(&CRegistryData::new(
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
        self.send_packet(&CUpdateTags::new(&tags)).await;

        // We are done with configuring
        self.send_packet(&CFinishConfig).await;

        if version < JavaMinecraftVersion::V_1_20_2 {
            return Some(self.handle_config_acknowledged(server).await);
        }

        debug!("Finished config");
        None
    }
}
