use indexmap::IndexMap;
use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};
use serde_json::Value;
use std::fs;

use crate::version::JavaMinecraftVersion;

/// The newest protocol version whose registry data is used as the fallback for unknown versions.
const LATEST_VERSION: JavaMinecraftVersion = JavaMinecraftVersion::V_26_3;

/// Generates the `TokenStream` for the `Registry` and `StaticRegistry` structs, version-keyed
/// static registry data, and the `Registry::get_synced` method.
pub(crate) fn build() -> TokenStream {
    let versions = [("26_3", "V_26_3")];

    const SYNCED_REGISTRIES: &[&str] = &[
        "worldgen/biome",
        "chat_type",
        "trim_pattern",
        "trim_material",
        "wolf_variant",
        "wolf_sound_variant",
        "pig_variant",
        "pig_sound_variant",
        "frog_variant",
        "cat_variant",
        "cat_sound_variant",
        "cow_variant",
        "cow_sound_variant",
        "chicken_variant",
        "chicken_sound_variant",
        "zombie_nautilus_variant",
        "painting_variant",
        "dimension_type",
        "damage_type",
        "jukebox_song",
        "banner_pattern",
        "instrument",
        "enchantment",
        "timeline",
        "dialog",
        "world_clock",
        "test_environment",
        "test_instance",
        "sulfur_cube_archetype",
        "decorated_pot_pattern",
        "block_transformer",
        "worldgen/block_state_provider",
    ];

    let process_version = |ver_folder: &str| -> TokenStream {
        let base_path = std::path::Path::new("../../assets/datapacks")
            .join(ver_folder)
            .join("data/minecraft");

        let mut data: IndexMap<String, IndexMap<String, Value>> = IndexMap::new();

        for &reg_name in SYNCED_REGISTRIES {
            let reg_dir = base_path.join(reg_name);
            if !reg_dir.is_dir() {
                continue;
            }
            let mut entries = IndexMap::new();
            let mut paths: Vec<_> = fs::read_dir(&reg_dir)
                .into_iter()
                .flatten()
                .filter_map(Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
                .collect();
            paths.sort_by_key(|e| e.path());

            for entry in paths {
                let path = entry.path();
                let stem = path.file_stem().unwrap().to_string_lossy().into_owned();
                if let Ok(content) = fs::read_to_string(&path)
                    && let Ok(val) = serde_json::from_str::<Value>(&content)
                {
                    entries.insert(stem, val);
                }
            }

            if !entries.is_empty() {
                data.insert(reg_name.to_string(), entries);
            }
        }

        // Inject "raw" chat type for vanilla parity
        if let Some(chat) = data.get_mut("chat_type") {
            chat.insert("raw".to_string(), serde_json::json!({
                "chat": { "translation_key": "%s", "parameters": ["content"] },
                "narration": { "translation_key": "%s says %s", "parameters": ["sender", "content"] }
            }));
        }

        let reg_tokens: Vec<TokenStream> = data
            .iter()
            .map(|(reg_name, entries)| {
                let entry_tokens: Vec<TokenStream> = entries
                    .iter()
                    .map(|(entry_name, entry_data)| {
                        fn json_to_nbt_tag(v: &Value) -> pumpkin_nbt::tag::NbtTag {
                            match v {
                                Value::Null => pumpkin_nbt::tag::NbtTag::End,
                                Value::Bool(b) => {
                                    pumpkin_nbt::tag::NbtTag::Byte(if *b { 1 } else { 0 })
                                }
                                Value::Number(num) => {
                                    if let Some(i) = num.as_i64() {
                                        if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
                                            pumpkin_nbt::tag::NbtTag::Int(i as i32)
                                        } else {
                                            pumpkin_nbt::tag::NbtTag::Long(i)
                                        }
                                    } else if let Some(f) = num.as_f64() {
                                        pumpkin_nbt::tag::NbtTag::Double(f)
                                    } else {
                                        pumpkin_nbt::tag::NbtTag::Int(0)
                                    }
                                }
                                Value::String(s) => {
                                    pumpkin_nbt::tag::NbtTag::String(s.clone().into())
                                }
                                Value::Array(arr) => pumpkin_nbt::tag::NbtTag::List(
                                    arr.iter().map(json_to_nbt_tag).collect(),
                                ),
                                Value::Object(obj) => {
                                    let mut compound = pumpkin_nbt::compound::NbtCompound::new();
                                    for (k, val) in obj {
                                        compound.put(k, json_to_nbt_tag(val));
                                    }
                                    pumpkin_nbt::tag::NbtTag::Compound(compound)
                                }
                            }
                        }

                        let nbt_tag = json_to_nbt_tag(entry_data);
                        let bytes = match nbt_tag {
                            pumpkin_nbt::tag::NbtTag::Compound(compound) => {
                                pumpkin_nbt::Nbt::from(compound).write_unnamed()
                            }
                            other => {
                                let mut bytes = Vec::new();
                                let mut writer =
                                    pumpkin_nbt::serializer::NbtWriteHelperJava::new(&mut bytes);
                                let _ = other.serialize(&mut writer);
                                bytes.into()
                            }
                        };
                        let byte_literal = Literal::byte_string(&bytes);

                        quote! {
                            StaticRegistryEntry {
                                name: #entry_name,
                                data: #byte_literal
                            }
                        }
                    })
                    .collect();

                quote! {
                    StaticRegistry {
                        registry_id: #reg_name,
                        entries: &[#(#entry_tokens),*],
                    }
                }
            })
            .collect();

        quote! { &[#(#reg_tokens),*] }
    };

    let mut static_values = TokenStream::new();
    for (ver_folder, ident_str) in versions {
        let registries = process_version(ver_folder);
        let ident = format_ident!("REGISTRY_{ident_str}");

        static_values.extend(quote! {
            pub static #ident: &[StaticRegistry] = #registries;
        });
    }

    quote! {
        use pumpkin_util::resource_location::ResourceLocation;
        use pumpkin_util::version::JavaMinecraftVersion;

        pub struct StaticRegistryEntry {
            pub name: &'static str,
            pub data: &'static [u8],
        }

        pub struct StaticRegistry {
            pub registry_id: &'static str,
            pub entries: &'static [StaticRegistryEntry],
        }

        pub struct RegistryEntryData {
            pub entry_id: ResourceLocation,
            pub data: Option<Box<[u8]>>,
        }

        pub struct Registry {
            pub registry_id: ResourceLocation,
            pub registry_entries: Vec<RegistryEntryData>,
        }

        #static_values

        impl Registry {
            #[must_use]
            pub fn get_synced(_version: JavaMinecraftVersion) -> Vec<Self> {
                let static_regs = REGISTRY_V_26_3;

                static_regs.iter().map(|static_reg| {
                    let registry_id = if static_reg.registry_id.contains(':') {
                        static_reg.registry_id.to_string()
                    } else {
                        format!("minecraft:{}", static_reg.registry_id)
                    };

                    let registry_entries = static_reg.entries.iter().map(|entry| {
                        let entry_id = format!("minecraft:{}", entry.name);

                        RegistryEntryData {
                            entry_id,
                            // Data is now sourced directly from the entry
                            data: Some(entry.data.to_vec().into_boxed_slice()),
                        }
                    }).collect();

                    Self { registry_id, registry_entries }
                }).collect()
            }
        }
    }
}
