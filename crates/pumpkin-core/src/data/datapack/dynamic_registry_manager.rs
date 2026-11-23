use std::collections::HashMap;
use std::sync::Arc;

use arc_swap::ArcSwap;
use indexmap::IndexMap;

use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_data::registry::{Registry, RegistryEntryData};
use pumpkin_util::version::JavaMinecraftVersion;

/// Represents a single dynamic registry holding all its entries in canonical network order.
#[derive(Clone, Debug, Default)]
pub struct DynamicRegistry {
    pub registry_id: String,
    pub entries: Vec<RegistryEntryData>,
    pub id_to_index: HashMap<String, usize>,
}

impl DynamicRegistry {
    #[must_use]
    pub fn new(registry_id: String, entries: Vec<RegistryEntryData>) -> Self {
        let mut id_to_index = HashMap::with_capacity(entries.len() * 2);
        for (i, entry) in entries.iter().enumerate() {
            id_to_index.insert(entry.entry_id.clone(), i);
            if let Some(stripped) = entry.entry_id.strip_prefix("minecraft:") {
                id_to_index.insert(stripped.to_string(), i);
            } else if !entry.entry_id.contains(':') {
                id_to_index.insert(format!("minecraft:{}", entry.entry_id), i);
            }
        }
        Self {
            registry_id,
            entries,
            id_to_index,
        }
    }

    #[must_use]
    pub fn get_raw_id(&self, entry_id: &str) -> Option<u32> {
        if let Some(&idx) = self.id_to_index.get(entry_id) {
            return u32::try_from(idx).ok();
        }
        if let Some(stripped) = entry_id.strip_prefix("minecraft:") {
            if let Some(&idx) = self.id_to_index.get(stripped) {
                return u32::try_from(idx).ok();
            }
        } else if let Some(&idx) = self.id_to_index.get(&format!("minecraft:{entry_id}")) {
            return u32::try_from(idx).ok();
        }
        None
    }

    #[must_use]
    pub fn get_entry_id(&self, raw_id: u32) -> Option<&str> {
        self.entries
            .get(raw_id as usize)
            .map(|e| e.entry_id.as_str())
    }

    #[must_use]
    pub fn get_entry_data(&self, entry_id: &str) -> Option<&[u8]> {
        let idx = self.get_raw_id(entry_id)? as usize;
        self.entries.get(idx).and_then(|e| e.data.as_deref())
    }

    #[must_use]
    pub fn get_entry_data_by_raw_id(&self, raw_id: u32) -> Option<&[u8]> {
        self.entries
            .get(raw_id as usize)
            .and_then(|e| e.data.as_deref())
    }

    #[must_use]
    pub fn contains(&self, entry_id: &str) -> bool {
        if self.id_to_index.contains_key(entry_id) {
            return true;
        }
        entry_id.strip_prefix("minecraft:").map_or_else(
            || {
                self.id_to_index
                    .contains_key(&format!("minecraft:{entry_id}"))
            },
            |stripped| self.id_to_index.contains_key(stripped),
        )
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

pub(crate) type RegistrySnapshot = IndexMap<String, Arc<DynamicRegistry>>;

/// Manages all dynamic registries (vanilla base registries and datapack additions/overrides).
/// Provides fast bidirectional lookup between resource IDs and raw network IDs.
#[derive(Debug)]
pub struct DynamicRegistryManager {
    registries: ArcSwap<RegistrySnapshot>,
    base_registries: Arc<RegistrySnapshot>,
}

impl Default for DynamicRegistryManager {
    fn default() -> Self {
        Self::new(CURRENT_MC_VERSION)
    }
}

impl DynamicRegistryManager {
    #[must_use]
    pub fn new(version: JavaMinecraftVersion) -> Self {
        let registries: RegistrySnapshot = Registry::get_synced(version)
            .into_iter()
            .map(|reg| {
                let id = reg.registry_id;
                let registry = DynamicRegistry::new(id.clone(), reg.registry_entries);
                (id, Arc::new(registry))
            })
            .collect();
        let base_registries = Arc::new(registries);
        Self {
            registries: ArcSwap::from(base_registries.clone()),
            base_registries,
        }
    }

    /// Merges custom datapack entries into the dynamic registries.
    /// Existing entries update in-place preserving their network ID;
    /// new entries are appended sequentially; new registries are registered.
    pub fn apply_custom_registries<S1: std::hash::BuildHasher, S2: std::hash::BuildHasher>(
        &self,
        custom_registries: &HashMap<String, HashMap<String, RegistryEntryData, S1>, S2>,
    ) {
        let live = self.registries.load_full();
        let mut registries = (*self.base_registries).clone();
        // Without client reconfiguration, removed entries must retain their network slots.
        for (id, existing) in live.iter().filter(|(_, registry)| !registry.is_empty()) {
            let mut entries = existing.entries.clone();
            if let Some(base) = self.base_registries.get(id) {
                entries[..base.entries.len()].clone_from_slice(&base.entries);
            }
            registries.insert(
                id.clone(),
                Arc::new(DynamicRegistry::new(id.clone(), entries)),
            );
        }
        let mut custom: Vec<_> = custom_registries.iter().collect();
        custom.sort_unstable_by_key(|(id, _)| *id);
        for (raw_reg_id, custom_entries) in custom {
            let reg_id = if raw_reg_id.contains(':') {
                raw_reg_id.clone()
            } else {
                format!("minecraft:{raw_reg_id}")
            };

            let entries = if let Some(existing) = registries.get(&reg_id) {
                if custom_entries.is_empty() {
                    continue;
                }
                super::dynamic_registry_loader::merge_dynamic_registry_entries(
                    &existing.entries,
                    custom_entries,
                )
            } else {
                let mut entries: Vec<_> = custom_entries.values().cloned().collect();
                entries.sort_unstable_by(|a, b| a.entry_id.cmp(&b.entry_id));
                entries
            };
            let registry = DynamicRegistry::new(reg_id.clone(), entries);
            registries.insert(reg_id, Arc::new(registry));
        }
        self.registries.store(Arc::new(registries));
    }

    fn find_registry<'a>(
        reg_guard: &'a RegistrySnapshot,
        registry_id: &str,
    ) -> Option<&'a Arc<DynamicRegistry>> {
        if let Some(reg) = reg_guard.get(registry_id) {
            return Some(reg);
        }
        registry_id.strip_prefix("minecraft:").map_or_else(
            || reg_guard.get(&format!("minecraft:{registry_id}")),
            |stripped| reg_guard.get(stripped),
        )
    }

    #[must_use]
    pub fn get_raw_id(&self, registry_id: &str, entry_id: &str) -> Option<u32> {
        let reg_guard = self.registries.load();
        let reg = Self::find_registry(&reg_guard, registry_id)?;
        reg.get_raw_id(entry_id)
    }

    #[must_use]
    pub fn get_entry_id(&self, registry_id: &str, raw_id: u32) -> Option<String> {
        let reg_guard = self.registries.load();
        let reg = Self::find_registry(&reg_guard, registry_id)?;
        reg.get_entry_id(raw_id).map(str::to_string)
    }

    #[must_use]
    pub fn get_entry_data(&self, registry_id: &str, entry_id: &str) -> Option<Box<[u8]>> {
        let reg_guard = self.registries.load();
        let reg = Self::find_registry(&reg_guard, registry_id)?;
        reg.get_entry_data(entry_id)
            .map(|b| b.to_vec().into_boxed_slice())
    }

    #[must_use]
    pub fn contains_entry(&self, registry_id: &str, entry_id: &str) -> bool {
        let reg_guard = self.registries.load();
        Self::find_registry(&reg_guard, registry_id).is_some_and(|r| r.contains(entry_id))
    }

    #[must_use]
    pub fn get_registry_entries(&self, registry_id: &str) -> Vec<RegistryEntryData> {
        let reg_guard = self.registries.load();
        Self::find_registry(&reg_guard, registry_id).map_or_else(Vec::new, |r| r.entries.clone())
    }

    #[must_use]
    pub fn get_all_entry_ids(&self, registry_id: &str) -> Vec<String> {
        let reg_guard = self.registries.load();
        Self::find_registry(&reg_guard, registry_id).map_or_else(Vec::new, |r| {
            r.entries.iter().map(|e| e.entry_id.clone()).collect()
        })
    }

    #[must_use]
    pub fn get_synced_registries(&self) -> Vec<Registry> {
        self.registries
            .load()
            .values()
            .map(|reg| Registry {
                registry_id: reg.registry_id.clone(),
                registry_entries: reg.entries.clone(),
            })
            .collect()
    }

    pub(crate) fn synced_snapshot(&self) -> Arc<RegistrySnapshot> {
        self.registries.load_full()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dynamic_registry_manager_initializes_with_vanilla() {
        let manager = DynamicRegistryManager::new(CURRENT_MC_VERSION);
        let synced = manager.get_synced_registries();
        assert!(synced.len() >= 30);

        // Verify painting_variant has alban at raw_id 0
        let alban_id = manager.get_raw_id("minecraft:painting_variant", "alban");
        assert_eq!(alban_id, Some(0));
        let alban_full = manager.get_raw_id("minecraft:painting_variant", "minecraft:alban");
        assert_eq!(alban_full, Some(0));

        let name = manager.get_entry_id("minecraft:painting_variant", 0);
        assert_eq!(name, Some("minecraft:alban".to_string()));

        // Verify wolf_variant has ashen at raw_id 0 and pale at raw_id 3
        let ashen_id = manager.get_raw_id("minecraft:wolf_variant", "ashen");
        assert_eq!(ashen_id, Some(0));
        let pale_id = manager.get_raw_id("minecraft:wolf_variant", "pale");
        assert_eq!(pale_id, Some(3));
    }

    #[test]
    fn dynamic_registry_manager_applies_custom_entries() {
        let manager = DynamicRegistryManager::new(CURRENT_MC_VERSION);
        let base_count = manager
            .get_registry_entries("minecraft:painting_variant")
            .len();

        let mut custom = HashMap::new();
        let mut custom_paintings = HashMap::new();
        custom_paintings.insert(
            "custom:mona_lisa".to_string(),
            RegistryEntryData {
                entry_id: "custom:mona_lisa".to_string(),
                data: Some(vec![1, 2, 3].into_boxed_slice()),
            },
        );
        custom.insert("minecraft:painting_variant".to_string(), custom_paintings);

        manager.apply_custom_registries(&custom);

        let new_entries = manager.get_registry_entries("minecraft:painting_variant");
        assert_eq!(new_entries.len(), base_count + 1);

        let custom_id = manager.get_raw_id("minecraft:painting_variant", "custom:mona_lisa");
        assert_eq!(custom_id, Some(base_count as u32));

        let custom_name = manager.get_entry_id("minecraft:painting_variant", base_count as u32);
        assert_eq!(custom_name, Some("custom:mona_lisa".to_string()));

        let custom_data = manager.get_entry_data("minecraft:painting_variant", "custom:mona_lisa");
        assert_eq!(custom_data, Some(vec![1, 2, 3].into_boxed_slice()));
    }

    #[test]
    fn login_snapshot_survives_datapack_reload() {
        let manager = DynamicRegistryManager::default();
        let mut custom = HashMap::new();
        for name in ["example:z", "example:a"] {
            custom.insert(name.to_owned(), HashMap::new());
        }
        manager.apply_custom_registries(&custom);
        let login = manager.synced_snapshot();
        custom.clear();
        manager.apply_custom_registries(&custom);

        let mut order = login.keys().rev();
        assert_eq!(order.next().map(String::as_str), Some("example:z"));
        assert_eq!(order.next().map(String::as_str), Some("example:a"));
        assert!(!manager.synced_snapshot().contains_key("example:z"));
    }

    #[test]
    fn reload_preserves_live_custom_network_ids() {
        let manager = DynamicRegistryManager::default();
        let registry = "minecraft:painting_variant".to_string();
        let mut entries = HashMap::new();
        for name in ["custom:b", "custom:c"] {
            entries.insert(
                name.to_string(),
                RegistryEntryData {
                    entry_id: name.to_string(),
                    data: None,
                },
            );
        }
        let mut custom = HashMap::from([(registry.clone(), entries)]);
        manager.apply_custom_registries(&custom);
        let b = manager.get_raw_id(&registry, "custom:b");
        let c = manager.get_raw_id(&registry, "custom:c");
        let login = manager.synced_snapshot();
        custom.get_mut(&registry).unwrap().insert(
            "custom:a".into(),
            RegistryEntryData {
                entry_id: "custom:a".into(),
                data: None,
            },
        );
        manager.apply_custom_registries(&custom);
        assert_eq!(manager.get_raw_id(&registry, "custom:b"), b);
        assert_eq!(manager.get_raw_id(&registry, "custom:c"), c);
        custom.get_mut(&registry).unwrap().remove("custom:b");
        manager.apply_custom_registries(&custom);
        assert_eq!(manager.get_raw_id(&registry, "custom:c"), c);
        manager
            .apply_custom_registries(&HashMap::<String, HashMap<String, RegistryEntryData>>::new());
        assert_eq!(manager.get_raw_id(&registry, "custom:c"), c);
        assert_eq!(
            manager.get_entry_id(&registry, b.unwrap()).as_deref(),
            Some("custom:b")
        );
        assert_eq!(login[&registry].get_raw_id("custom:c"), c);
    }

    #[test]
    fn dynamic_registry_manager_creates_new_custom_registry() {
        let manager = DynamicRegistryManager::new(CURRENT_MC_VERSION);

        let mut custom = HashMap::new();
        let mut custom_magic = HashMap::new();
        custom_magic.insert(
            "custom:fire_spell".to_string(),
            RegistryEntryData {
                entry_id: "custom:fire_spell".to_string(),
                data: None,
            },
        );
        custom.insert("custom:spells".to_string(), custom_magic);

        manager.apply_custom_registries(&custom);

        let spell_id = manager.get_raw_id("custom:spells", "custom:fire_spell");
        assert_eq!(spell_id, Some(0));

        let synced = manager.get_synced_registries();
        assert!(synced.iter().any(|r| r.registry_id == "custom:spells"));
    }
}
