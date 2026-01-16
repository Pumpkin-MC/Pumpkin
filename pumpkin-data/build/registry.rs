use indexmap::IndexMap;
use proc_macro2::{Literal, TokenStream};
use quote::quote;
use serde_json::Value;
use std::fs;

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/synced_registries.json");

    let json_str = fs::read_to_string("../assets/synced_registries.json")
        .expect("Failed to read synced_registries.json");
    let mut data: IndexMap<String, IndexMap<String, Value>> =
        serde_json::from_str(&json_str).expect("Failed to parse synced_registries.json");

    let chat_type = if data.contains_key("minecraft:chat_type") {
        data.get_mut("minecraft:chat_type")
    } else {
        data.get_mut("chat_type")
    };

    if let Some(chat_type) = chat_type {
        chat_type.insert(
            "raw".to_string(),
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
            }),
        );
    }

    let registries_tokens = data.iter().map(|(reg_name, entries)| {
        let entry_tokens = entries.iter().map(|(entry_name, entry_data)| {
            let mut nbt_bytes = Vec::new();
            pumpkin_nbt::serializer::to_bytes_unnamed(entry_data, &mut nbt_bytes)
                .expect("Failed to serialize NBT at build time");

            let nbt_literal = Literal::byte_string(&nbt_bytes);

            quote! {
                StaticRegistryEntry {
                    name: #entry_name,
                    nbt_bytes: #nbt_literal,
                }
            }
        });

        quote! {
            StaticRegistry {
                registry_id: #reg_name,
                entries: &[#(#entry_tokens),*],
            }
        }
    });

    let jukebox_entries = data
        .get("minecraft:jukebox_song")
        .or_else(|| data.get("jukebox_song"));

    let jukebox_index_arms = jukebox_entries
        .map(|entries| {
            entries
                .iter()
                .enumerate()
                .map(|(index, (name, _))| {
                    quote! { #name => Some(#index), }
                })
                .collect::<TokenStream>()
        })
        .unwrap_or_default();

    quote! {
        use pumpkin_util::resource_location::ResourceLocation;

        pub struct StaticRegistryEntry {
            pub name: &'static str,
            pub nbt_bytes: &'static [u8],
        }

        pub struct StaticRegistry {
            pub registry_id: &'static str,
            pub entries: &'static [StaticRegistryEntry],
        }

        pub static SYNCED_REGISTRIES: &[StaticRegistry] = &[
            #(#registries_tokens),*
        ];

        pub struct RegistryEntryData {
            pub entry_id: ResourceLocation,
            pub data: Option<Box<[u8]>>,
        }

        pub struct Registry {
            pub registry_id: ResourceLocation,
            pub registry_entries: Vec<RegistryEntryData>,
        }

        pub struct SyncedRegistry;

        impl SyncedRegistry {
            pub fn get_jukebox_song_index(song_key: &str) -> Option<usize> {
                match song_key {
                    #jukebox_index_arms
                    _ => None
                }
            }
        }

        impl Registry {
            pub fn get_synced() -> Vec<Self> {
                SYNCED_REGISTRIES.iter().map(|static_reg| {
                    let registry_id = if static_reg.registry_id.contains(':') {
                        ResourceLocation::from(static_reg.registry_id)
                    } else {
                        ResourceLocation::vanilla(static_reg.registry_id)
                    };

                    let registry_entries = static_reg.entries.iter().map(|entry| {
                        let entry_id = if entry.name.contains(':') {
                            ResourceLocation::from(entry.name)
                        } else {
                            ResourceLocation::vanilla(entry.name)
                        };

                        let data = Some(entry.nbt_bytes.to_vec().into_boxed_slice());

                        RegistryEntryData {
                            entry_id,
                            data,
                        }
                    }).collect();

                    Self {
                        registry_id,
                        registry_entries,
                    }
                }).collect()
            }
        }
    }
}
