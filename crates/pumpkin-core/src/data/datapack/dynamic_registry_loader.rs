use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use pumpkin_data::registry::RegistryEntryData;
use pumpkin_nbt::Nbt;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use rayon::prelude::*;
use serde_json::Value;
use tracing::warn;

use super::file_loader::{collect_files, load_files_parallel};

pub const DYNAMIC_REGISTRIES: &[(&str, &[&str])] = &[
    ("minecraft:wolf_variant", &["wolf_variant", "wolf_variants"]),
    ("minecraft:cat_variant", &["cat_variant", "cat_variants"]),
    ("minecraft:frog_variant", &["frog_variant", "frog_variants"]),
    ("minecraft:instrument", &["instrument", "instruments"]),
    (
        "minecraft:trim_material",
        &["trim_material", "trim_materials"],
    ),
    ("minecraft:trim_pattern", &["trim_pattern", "trim_patterns"]),
    (
        "minecraft:banner_pattern",
        &["banner_pattern", "banner_patterns"],
    ),
    (
        "minecraft:decorated_pot_pattern",
        &["decorated_pot_pattern", "decorated_pot_patterns"],
    ),
    ("minecraft:chat_type", &["chat_type", "chat_types"]),
    ("minecraft:enchantment", &["enchantment", "enchantments"]),
    (
        "minecraft:enchantment_provider",
        &["enchantment_provider", "enchantment_providers"],
    ),
    ("minecraft:cow_variant", &["cow_variant", "cow_variants"]),
    (
        "minecraft:cow_sound_variant",
        &["cow_sound_variant", "cow_sound_variants"],
    ),
    ("minecraft:pig_variant", &["pig_variant", "pig_variants"]),
    (
        "minecraft:pig_sound_variant",
        &["pig_sound_variant", "pig_sound_variants"],
    ),
    (
        "minecraft:chicken_variant",
        &["chicken_variant", "chicken_variants"],
    ),
    (
        "minecraft:chicken_sound_variant",
        &["chicken_sound_variant", "chicken_sound_variants"],
    ),
    (
        "minecraft:cat_sound_variant",
        &["cat_sound_variant", "cat_sound_variants"],
    ),
    (
        "minecraft:wolf_sound_variant",
        &["wolf_sound_variant", "wolf_sound_variants"],
    ),
    (
        "minecraft:zombie_nautilus_variant",
        &["zombie_nautilus_variant", "zombie_nautilus_variants"],
    ),
    ("minecraft:trade_set", &["trade_set", "trade_sets"]),
    (
        "minecraft:villager_trade",
        &["villager_trade", "villager_trades"],
    ),
    (
        "minecraft:painting_variant",
        &["painting_variant", "painting_variants"],
    ),
    ("minecraft:dimension", &["dimension", "dimensions"]),
    (
        "minecraft:dimension_type",
        &["dimension_type", "dimension_types"],
    ),
    (
        "minecraft:item_modifier",
        &["item_modifier", "item_modifiers"],
    ),
    ("minecraft:slot_source", &["slot_source", "slot_sources"]),
    ("minecraft:jukebox_song", &["jukebox_song", "jukebox_songs"]),
    ("minecraft:dialog", &["dialog", "dialogs"]),
    ("minecraft:timeline", &["timeline", "timelines"]),
    ("minecraft:world_clock", &["world_clock", "world_clocks"]),
    (
        "minecraft:test_environment",
        &["test_environment", "test_environments"],
    ),
    (
        "minecraft:test_instance",
        &["test_instance", "test_instances"],
    ),
    (
        "minecraft:sulfur_cube_archetype",
        &["sulfur_cube_archetype", "sulfur_cube_archetypes"],
    ),
    (
        "minecraft:block_transformer",
        &["block_transformer", "block_transformers"],
    ),
    (
        "minecraft:block_sound_set",
        &["block_sound_set", "block_sound_sets"],
    ),
    (
        "minecraft:trial_spawner",
        &["trial_spawner", "trial_spawners"],
    ),
    (
        "minecraft:worldgen/biome",
        &["worldgen/biome", "worldgen/biomes"],
    ),
    (
        "minecraft:worldgen/block_state_provider",
        &[
            "worldgen/block_state_provider",
            "worldgen/block_state_providers",
        ],
    ),
    (
        "minecraft:worldgen/carver",
        &[
            "worldgen/carver",
            "worldgen/carvers",
            "worldgen/configured_carver",
            "worldgen/configured_carvers",
        ],
    ),
    (
        "minecraft:worldgen/configured_carver",
        &[
            "worldgen/configured_carver",
            "worldgen/configured_carvers",
            "worldgen/carver",
            "worldgen/carvers",
        ],
    ),
    (
        "minecraft:worldgen/density_function",
        &["worldgen/density_function", "worldgen/density_functions"],
    ),
    (
        "minecraft:worldgen/feature",
        &[
            "worldgen/feature",
            "worldgen/features",
            "worldgen/configured_feature",
            "worldgen/configured_features",
        ],
    ),
    (
        "minecraft:worldgen/configured_feature",
        &[
            "worldgen/configured_feature",
            "worldgen/configured_features",
            "worldgen/feature",
            "worldgen/features",
        ],
    ),
    (
        "minecraft:worldgen/flat_level_generator_preset",
        &[
            "worldgen/flat_level_generator_preset",
            "worldgen/flat_level_generator_presets",
        ],
    ),
    (
        "minecraft:worldgen/material_condition",
        &[
            "worldgen/material_condition",
            "worldgen/material_conditions",
        ],
    ),
    (
        "minecraft:worldgen/material_rule",
        &["worldgen/material_rule", "worldgen/material_rules"],
    ),
    (
        "minecraft:worldgen/multi_noise_biome_source_parameter_list",
        &[
            "worldgen/multi_noise_biome_source_parameter_list",
            "worldgen/multi_noise_biome_source_parameter_lists",
        ],
    ),
    (
        "minecraft:worldgen/noise",
        &["worldgen/noise", "worldgen/noises"],
    ),
    (
        "minecraft:worldgen/noise_settings",
        &["worldgen/noise_settings"],
    ),
    (
        "minecraft:worldgen/placed_feature",
        &["worldgen/placed_feature", "worldgen/placed_features"],
    ),
    (
        "minecraft:worldgen/processor_list",
        &["worldgen/processor_list", "worldgen/processor_lists"],
    ),
    (
        "minecraft:worldgen/structure",
        &[
            "worldgen/structure",
            "worldgen/structures",
            "worldgen/configured_structure",
            "worldgen/configured_structures",
        ],
    ),
    (
        "minecraft:worldgen/structure_set",
        &["worldgen/structure_set", "worldgen/structure_sets"],
    ),
    (
        "minecraft:worldgen/template_pool",
        &["worldgen/template_pool", "worldgen/template_pools"],
    ),
    (
        "minecraft:worldgen/world_preset",
        &["worldgen/world_preset", "worldgen/world_presets"],
    ),
];

/// Converts a JSON `serde_json::Value` into an `NbtTag`.
#[must_use]
pub fn json_to_nbt_tag(v: &Value) -> NbtTag {
    match v {
        Value::Null => NbtTag::End,
        Value::Bool(b) => NbtTag::Byte(i8::from(*b)),
        Value::Number(num) => num.as_i64().map_or_else(
            || num.as_f64().map_or(NbtTag::Int(0), NbtTag::Double),
            |i| {
                if i32::try_from(i).is_ok() {
                    NbtTag::Int(i as i32)
                } else {
                    NbtTag::Long(i)
                }
            },
        ),
        Value::String(s) => NbtTag::String(s.clone().into()),
        Value::Array(arr) => NbtTag::List(arr.iter().map(json_to_nbt_tag).collect()),
        Value::Object(obj) => {
            let mut compound = NbtCompound::new();
            for (k, val) in obj {
                compound.put(k, json_to_nbt_tag(val));
            }
            NbtTag::Compound(compound)
        }
    }
}

/// Encodes a JSON definition into binary unnamed NBT bytes.
#[must_use]
pub fn json_to_registry_nbt_bytes(val: &Value) -> Box<[u8]> {
    let nbt_tag = json_to_nbt_tag(val);
    match nbt_tag {
        NbtTag::Compound(compound) => Nbt::from(compound)
            .write_unnamed()
            .to_vec()
            .into_boxed_slice(),
        other => {
            let mut bytes = Vec::new();
            let mut writer = pumpkin_nbt::serializer::NbtWriteHelperJava::new(&mut bytes);
            let _ = other.serialize(&mut writer);
            bytes.into_boxed_slice()
        }
    }
}

fn load_registry_dir(namespace: &str, dir: &Path) -> Vec<RegistryEntryData> {
    let files = collect_files(dir, "json");
    load_files_parallel(namespace, dir, &files, "json", |_id, path, content| {
        match serde_json::from_str::<Value>(content) {
            Ok(val) => Some(json_to_registry_nbt_bytes(&val)),
            Err(error) => {
                warn!("Failed to parse registry entry {}: {error}", path.display());
                None
            }
        }
    })
    .into_iter()
    .map(|(entry_id, _, bytes)| RegistryEntryData {
        entry_id,
        data: Some(bytes),
    })
    .collect()
}

const EXCLUDED_REGISTRY_DIRS: &[&str] = &[
    "recipe",
    "recipes",
    "function",
    "functions",
    "tags",
    "structure",
    "structures",
    "advancement",
    "advancements",
    "loot_table",
    "loot_tables",
    "predicate",
    "predicates",
    "item_modifier",
    "item_modifiers",
    "slot_source",
    "slot_sources",
    "context_int_provider",
    "context_int_providers",
    "context_float_provider",
    "context_float_providers",
    "damage_type",
    "damage_types",
    "datapacks",
];

fn registry_directories(namespace: &Path, prefix: &Path, dirs: &mut BTreeSet<PathBuf>) {
    let dir = namespace.join(prefix);
    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(error) => {
            warn!(
                "Failed to read registry directory {}: {error}",
                dir.display()
            );
            return;
        }
    };
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                warn!(
                    "Failed to read directory entry in {}: {error}",
                    dir.display()
                );
                continue;
            }
        };
        if !entry.path().is_dir() {
            continue;
        }
        let relative = prefix.join(entry.file_name());
        if relative == Path::new("worldgen") {
            registry_directories(namespace, &relative, dirs);
        } else {
            dirs.insert(relative);
        }
    }
}

/// Scans a namespace directory for any dynamic registry subfolders and loads their JSON definitions.
pub fn load_dynamic_registries_from_ns<
    S1: std::hash::BuildHasher + Default,
    S2: std::hash::BuildHasher,
>(
    namespace: &str,
    ns_path: &Path,
    registries: &mut HashMap<String, HashMap<String, RegistryEntryData, S1>, S2>,
) -> usize {
    let mut dirs = BTreeSet::new();
    registry_directories(ns_path, Path::new(""), &mut dirs);
    let mut uses = HashMap::new();
    let mut jobs = Vec::new();
    for &(registry_id, aliases) in DYNAMIC_REGISTRIES {
        // Empty known registries are included in the network registry list.
        registries.entry(registry_id.to_owned()).or_default();
        let paths: Vec<_> = aliases
            .iter()
            .filter_map(|name| dirs.get(Path::new(name)))
            .collect();
        if !paths.is_empty() {
            for &path in &paths {
                *uses.entry(path).or_insert(0usize) += 1;
            }
            jobs.push((registry_id.to_owned(), paths));
        }
    }
    for dir in &dirs {
        if uses.contains_key(dir)
            || EXCLUDED_REGISTRY_DIRS
                .iter()
                .any(|name| dir == Path::new(name))
        {
            continue;
        }
        let path = dir
            .to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "/");
        uses.insert(dir, 1);
        jobs.push((format!("{namespace}:{path}"), vec![dir]));
    }

    // Several registries share directory aliases with different override orders.
    // Parse each directory once, then apply each registry's ordered list.
    let mut prepared: HashMap<_, _> = uses
        .into_par_iter()
        .map(|(path, uses)| {
            (
                path,
                (uses, load_registry_dir(namespace, &ns_path.join(path))),
            )
        })
        .collect();
    let mut total_loaded = 0;
    for (id, paths) in jobs {
        let registry = registries.entry(id).or_default();
        let before = registry.len();
        for path in paths {
            let Some((remaining, entries)) = prepared.get_mut(path) else {
                continue;
            };
            *remaining -= 1;
            let entries = if *remaining == 0 {
                std::mem::take(entries)
            } else {
                entries.clone()
            };
            registry.extend(
                entries
                    .into_iter()
                    .map(|entry| (entry.entry_id.clone(), entry)),
            );
        }
        total_loaded += registry.len() - before;
    }
    total_loaded
}

/// Merges vanilla registry entries with custom datapack entries for a registry.
///
/// Vanilla entries are preserved in their original order, unless overridden by a custom entry
/// with matching entry ID. Custom entries not matching any vanilla entry are sorted
/// lexicographically and appended to the end.
#[must_use]
pub fn merge_dynamic_registry_entries<S: std::hash::BuildHasher>(
    vanilla_entries: &[RegistryEntryData],
    custom_entries: &HashMap<String, RegistryEntryData, S>,
) -> Vec<RegistryEntryData> {
    if custom_entries.is_empty() {
        return vanilla_entries
            .iter()
            .map(|entry| RegistryEntryData {
                entry_id: entry.entry_id.clone(),
                data: entry.data.clone(),
            })
            .collect();
    }

    let mut merged = Vec::with_capacity(vanilla_entries.len() + custom_entries.len());
    let mut overridden = HashSet::new();

    for entry in vanilla_entries {
        let full_id = if entry.entry_id.contains(':') {
            entry.entry_id.clone()
        } else {
            format!("minecraft:{}", entry.entry_id)
        };

        if let Some(custom) = custom_entries
            .get(&full_id)
            .or_else(|| custom_entries.get(&entry.entry_id))
        {
            let mut reg_entry = RegistryEntryData {
                entry_id: entry.entry_id.clone(),
                data: custom.data.clone(),
            };
            reg_entry.entry_id.clone_from(&entry.entry_id);
            merged.push(reg_entry);
            overridden.insert(full_id);
            overridden.insert(entry.entry_id.clone());
        } else {
            merged.push(RegistryEntryData {
                entry_id: entry.entry_id.clone(),
                data: entry.data.clone(),
            });
        }
    }

    let mut sorted_custom: Vec<&RegistryEntryData> = custom_entries
        .values()
        .filter(|e| !overridden.contains(&e.entry_id))
        .collect();
    sorted_custom.sort_by(|a, b| a.entry_id.cmp(&b.entry_id));

    for custom in sorted_custom {
        merged.push(RegistryEntryData {
            entry_id: custom.entry_id.clone(),
            data: custom.data.clone(),
        });
    }

    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_directory_aliases_keep_registry_override_order() {
        let dir = tempfile::tempdir().unwrap();
        for (folder, value) in [("feature", 1), ("configured_feature", 2)] {
            let path = dir.path().join("worldgen").join(folder);
            fs::create_dir_all(&path).unwrap();
            fs::write(path.join("shared.json"), value.to_string()).unwrap();
        }
        let mut registries: HashMap<_, HashMap<_, _>> = HashMap::new();
        assert_eq!(
            load_dynamic_registries_from_ns("pack", dir.path(), &mut registries),
            2
        );
        for (registry, value) in [
            ("minecraft:worldgen/feature", 2),
            ("minecraft:worldgen/configured_feature", 1),
        ] {
            assert_eq!(
                registries[registry]["pack:shared"].data,
                Some(json_to_registry_nbt_bytes(&Value::from(value)))
            );
        }
    }

    #[test]
    fn json_to_nbt_roundtrip() {
        let json: Value = serde_json::json!({
            "asset_id": "custom:entity/cat/fluffy",
            "number": 42,
            "float": 3.25,
            "flag": true,
            "nested": {
                "tag": "value"
            },
            "list": ["a", "b"]
        });

        let bytes = json_to_registry_nbt_bytes(&json);
        assert!(!bytes.is_empty());

        let mut cursor = std::io::Cursor::new(&bytes[..]);
        let mut reader = pumpkin_nbt::deserializer::NbtReadHelperJava::new(
            pumpkin_nbt::deserializer::NbtStreamReader(&mut cursor),
        );
        let nbt = Nbt::read_unnamed(&mut reader).unwrap().root_tag;
        assert_eq!(nbt.get_string("asset_id"), Some("custom:entity/cat/fluffy"));
        assert_eq!(nbt.get_int("number"), Some(42));
        assert_eq!(nbt.get_byte("flag"), Some(1));
    }

    #[test]
    fn merge_preserves_and_appends() {
        let vanilla = vec![
            RegistryEntryData {
                entry_id: "minecraft:all_black".to_string(),
                data: Some(vec![1, 2, 3].into_boxed_slice()),
            },
            RegistryEntryData {
                entry_id: "minecraft:black".to_string(),
                data: Some(vec![4, 5, 6].into_boxed_slice()),
            },
        ];

        let mut custom = HashMap::new();
        custom.insert(
            "custom:fluffy".to_string(),
            RegistryEntryData {
                entry_id: "custom:fluffy".to_string(),
                data: Some(vec![7, 8, 9].into_boxed_slice()),
            },
        );
        custom.insert(
            "minecraft:black".to_string(),
            RegistryEntryData {
                entry_id: "minecraft:black".to_string(),
                data: Some(vec![99].into_boxed_slice()),
            },
        );

        let merged = merge_dynamic_registry_entries(&vanilla, &custom);
        assert_eq!(merged.len(), 3);
        assert_eq!(merged[0].entry_id, "minecraft:all_black");
        assert_eq!(merged[0].data.as_deref(), Some(&[1, 2, 3][..]));
        assert_eq!(merged[1].entry_id, "minecraft:black");
        assert_eq!(merged[1].data.as_deref(), Some(&[99][..]));
        assert_eq!(merged[2].entry_id, "custom:fluffy");
        assert_eq!(merged[2].data.as_deref(), Some(&[7, 8, 9][..]));
    }
}
