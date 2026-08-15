#[allow(clippy::wildcard_imports)]
use super::*;

impl PendingConnection {
    pub async fn handle_known_packs(&mut self, _packet: SKnownPacks<'_>, _server: &Server) {
        let version = self.version.load();
        if version >= JavaMinecraftVersion::V_1_20_2 {
            self.send_packet_now(&CFeatureFlags::new(&["minecraft:vanilla".to_string()]))
                .await;
            let registry = pumpkin_data::registry::Registry::get_synced(version);
            for reg in &registry {
                self.send_packet_now(&CRegistryData::new(&reg.registry_id, &reg.registry_entries))
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
        self.send_packet_now(&CFinishConfig).await;
    }
}
