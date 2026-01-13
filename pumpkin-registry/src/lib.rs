use std::sync::LazyLock;

mod jukebox_song;

use indexmap::IndexMap;
use pumpkin_protocol::java::client::config::RegistryEntry;
use pumpkin_util::include_json_static;
use pumpkin_util::resource_location::ResourceLocation;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::jukebox_song::JukeboxSong;

pub static SYNCED_REGISTRIES: LazyLock<SyncedRegistry> =
    LazyLock::new(|| include_json_static!("../../assets/synced_registries.json", SyncedRegistry));

pub struct Registry {
    pub registry_id: ResourceLocation,
    pub registry_entries: Vec<RegistryEntry>,
}

#[derive(Serialize, Deserialize)]
pub struct SyncedRegistry {
    #[serde(flatten)]
    pub registries: IndexMap<String, IndexMap<String, Value>>,
}

// Probably not optimal
impl SyncedRegistry {
    #[must_use] 
    pub fn get_jukebox_song(&self, song_key: &str) -> Option<JukeboxSong> {
        let jukebox_registry = self
            .registries
            .get("minecraft:jukebox_song")
            .or_else(|| self.registries.get("jukebox_song"))?;
        let song_value = jukebox_registry.get(song_key)?;

        serde_json::from_value(song_value.clone()).ok()
    }

    #[must_use] 
    pub fn get_jukebox_song_index(&self, song_key: &str) -> Option<usize> {
        let jukebox_registry = self
            .registries
            .get("minecraft:jukebox_song")
            .or_else(|| self.registries.get("jukebox_song"))?;
        jukebox_registry.get_index_of(song_key)
    }
}

impl Registry {
    #[must_use] 
    pub fn get_synced() -> Vec<Self> {
        let mut synced_registries = Vec::new();

        for (registry_key, entries) in &SYNCED_REGISTRIES.registries {
            let registry_id = if registry_key.contains(':') {
                ResourceLocation::from(registry_key)
            } else {
                ResourceLocation::vanilla(registry_key)
            };

            let mut registry_entries: Vec<RegistryEntry> = entries
                .iter()
                .map(|(entry_name, entry_data)| RegistryEntry::from_nbt(entry_name, entry_data))
                .collect();

            // 3. Special Handling: Chat Type "raw" injection
            // We check if this specific loop iteration is for "chat_type"
            if registry_id.path == "chat_type" {
                registry_entries.push(RegistryEntry::from_nbt_custom(
                    "raw",
                    &Self::get_raw_chat_type_struct(),
                ));
            }

            synced_registries.push(Self {
                registry_id,
                registry_entries,
            });
        }

        // let jukebox_entries = SYNCED_REGISTRIES
        //     .jukebox_song
        //     .keys()
        //     .map(|name| RegistryEntry::none(name))
        //     .collect();

        // synced_registries.push(Registry {
        //     registry_id: ResourceLocation::vanilla("jukebox_song"),
        //     registry_entries: jukebox_entries,
        // });

        synced_registries
    }

    fn get_raw_chat_type_struct() -> impl Serialize {
        serde_json::json!({
            "chat": {
                "translation_key": "%s",
                "parameters": ["content"],
                "style": null
            },
            "narration": {
                "translation_key": "%s says %s",
                "parameters": ["sender", "content"],
                "style": null
            }
        })
    }
}
