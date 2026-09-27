#[allow(clippy::wildcard_imports)]
use super::*;

impl PendingConnection {
    pub async fn handle_known_packs(&mut self, server: &Server) {
        let version = CURRENT_MC_VERSION;
        let registry = pumpkin_data::registry::Registry::get_synced(version);
        for reg in &registry {
            if reg.registry_id == "minecraft:damage_type" {
                let merged = server
                    .datapack_manager
                    .merge_damage_type_entries(&reg.registry_entries);
                self.send_packet_now(&CRegistryData::new(&reg.registry_id, &merged))
                    .await;
                continue;
            }
            let merged = server
                .datapack_manager
                .merge_registry_entries(&reg.registry_id, &reg.registry_entries);
            self.send_packet_now(&CRegistryData::new(&reg.registry_id, &merged))
                .await;
        }
        let mut tags = Vec::new();
        for &key in pumpkin_data::tag::RegistryKey::NETWORK_KEYS {
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
