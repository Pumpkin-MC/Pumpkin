use std::collections::HashMap;
use std::sync::RwLock;

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

/// Manages all dynamic registries (vanilla base registries and datapack additions/overrides).
/// Provides fast bidirectional lookup between resource IDs and raw network IDs.
#[derive(Debug)]
pub struct DynamicRegistryManager {
    registries: RwLock<HashMap<String, DynamicRegistry>>,
    synced_order: RwLock<Vec<String>>,
    base_registries: HashMap<String, DynamicRegistry>,
    base_synced_order: Vec<String>,
}

impl Default for DynamicRegistryManager {
    fn default() -> Self {
        Self::new(CURRENT_MC_VERSION)
    }
}

impl DynamicRegistryManager {
    #[must_use]
    pub fn new(version: JavaMinecraftVersion) -> Self {
        let mut registries = HashMap::new();
        let mut synced_order = Vec::new();

        let base_regs = pumpkin_data::registry::Registry::get_synced(version);
        for reg in base_regs {
            let reg_id = if reg.registry_id.contains(':') {
                reg.registry_id.clone()
            } else {
                format!("minecraft:{}", reg.registry_id)
            };
            let dyn_reg = DynamicRegistry::new(reg_id.clone(), reg.registry_entries);
            registries.insert(reg_id.clone(), dyn_reg);
            synced_order.push(reg_id);
        }

        Self {
            base_registries: registries.clone(),
            base_synced_order: synced_order.clone(),
            registries: RwLock::new(registries),
            synced_order: RwLock::new(synced_order),
        }
    }

    /// Merges custom datapack entries into the dynamic registries.
    /// Existing entries update in-place preserving their network ID;
    /// new entries are appended sequentially; new registries are registered.
    pub fn apply_custom_registries<S1: std::hash::BuildHasher, S2: std::hash::BuildHasher>(
        &self,
        custom_registries: &HashMap<String, HashMap<String, RegistryEntryData, S1>, S2>,
    ) {
        let mut reg_guard = self
            .registries
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut order_guard = self
            .synced_order
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        (*reg_guard).clone_from(&self.base_registries);
        (*order_guard).clone_from(&self.base_synced_order);

        for (raw_reg_id, custom_entries) in custom_registries {
            let reg_id = if raw_reg_id.contains(':') {
                raw_reg_id.clone()
            } else {
                format!("minecraft:{raw_reg_id}")
            };

            if let Some(existing) = reg_guard.get_mut(&reg_id) {
                let merged = super::dynamic_registry_loader::merge_dynamic_registry_entries(
                    &existing.entries,
                    custom_entries,
                );
                *existing = DynamicRegistry::new(reg_id.clone(), merged);
            } else {
                let mut entries: Vec<RegistryEntryData> = custom_entries
                    .values()
                    .map(|e| RegistryEntryData {
                        entry_id: e.entry_id.clone(),
                        data: e.data.clone(),
                    })
                    .collect();
                entries.sort_by(|a, b| a.entry_id.cmp(&b.entry_id));
                let dyn_reg = DynamicRegistry::new(reg_id.clone(), entries);
                reg_guard.insert(reg_id.clone(), dyn_reg);
                order_guard.push(reg_id);
            }
        }
    }

    fn find_registry<'a>(
        reg_guard: &'a HashMap<String, DynamicRegistry>,
        registry_id: &str,
    ) -> Option<&'a DynamicRegistry> {
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
        let reg_guard = self
            .registries
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let reg = Self::find_registry(&reg_guard, registry_id)?;
        reg.get_raw_id(entry_id)
    }

    #[must_use]
    pub fn get_entry_id(&self, registry_id: &str, raw_id: u32) -> Option<String> {
        let reg_guard = self
            .registries
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let reg = Self::find_registry(&reg_guard, registry_id)?;
        reg.get_entry_id(raw_id).map(str::to_string)
    }

    #[must_use]
    pub fn get_entry_data(&self, registry_id: &str, entry_id: &str) -> Option<Box<[u8]>> {
        let reg_guard = self
            .registries
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let reg = Self::find_registry(&reg_guard, registry_id)?;
        reg.get_entry_data(entry_id)
            .map(|b| b.to_vec().into_boxed_slice())
    }

    #[must_use]
    pub fn contains_entry(&self, registry_id: &str, entry_id: &str) -> bool {
        let reg_guard = self
            .registries
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Self::find_registry(&reg_guard, registry_id).is_some_and(|r| r.contains(entry_id))
    }

    #[must_use]
    pub fn get_registry_entries(&self, registry_id: &str) -> Vec<RegistryEntryData> {
        let reg_guard = self
            .registries
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Self::find_registry(&reg_guard, registry_id).map_or_else(Vec::new, |r| r.entries.clone())
    }

    #[must_use]
    pub fn get_all_entry_ids(&self, registry_id: &str) -> Vec<String> {
        let reg_guard = self
            .registries
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Self::find_registry(&reg_guard, registry_id).map_or_else(Vec::new, |r| {
            r.entries.iter().map(|e| e.entry_id.clone()).collect()
        })
    }

    #[must_use]
    pub fn get_synced_registries(&self) -> Vec<Registry> {
        let order_guard = self
            .synced_order
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let reg_guard = self
            .registries
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let mut result = Vec::with_capacity(order_guard.len());
        for id in order_guard.iter() {
            if let Some(dyn_reg) = reg_guard.get(id) {
                result.push(Registry {
                    registry_id: dyn_reg.registry_id.clone(),
                    registry_entries: dyn_reg.entries.clone(),
                });
            }
        }
        result
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
