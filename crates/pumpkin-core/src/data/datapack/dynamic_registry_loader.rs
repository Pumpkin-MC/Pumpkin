use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use pumpkin_data::registry::RegistryEntryData;
use pumpkin_nbt::Nbt;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use serde_json::Value;

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

/// Recursively loads JSON files from `current_dir` and inserts them into `registry_map`.
fn load_registry_dir_recursive<S: std::hash::BuildHasher>(
    namespace: &str,
    base_dir: &Path,
    current_dir: &Path,
    registry_map: &mut HashMap<String, RegistryEntryData, S>,
) {
    let Ok(entries) = fs::read_dir(current_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            load_registry_dir_recursive(namespace, base_dir, &path, registry_map);
        } else if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
            && let Ok(rel_path) = path.strip_prefix(base_dir)
        {
            let mut stem_path = rel_path.to_string_lossy().to_string();
            if let Some(stem) = stem_path.strip_suffix(".json") {
                stem_path = stem.to_string();
            }
            let stem_path = stem_path.replace('\\', "/");
            let entry_id = format!("{namespace}:{stem_path}");

            if let Ok(content) = fs::read_to_string(&path)
                && let Ok(val) = serde_json::from_str::<Value>(&content)
            {
                let nbt_bytes = json_to_registry_nbt_bytes(&val);
                registry_map.insert(
                    entry_id.clone(),
                    RegistryEntryData {
                        entry_id,
                        data: Some(nbt_bytes),
                    },
                );
            }
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
    let mut total_loaded = 0;

    for &(registry_id, dir_names) in DYNAMIC_REGISTRIES {
        let registry_map = registries.entry(registry_id.to_string()).or_default();
        for &dir_name in dir_names {
            let reg_dir = ns_path.join(dir_name);
            if reg_dir.is_dir() {
                let before = registry_map.len();
                load_registry_dir_recursive(namespace, &reg_dir, &reg_dir, registry_map);
                total_loaded += registry_map.len() - before;
            }
        }
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
