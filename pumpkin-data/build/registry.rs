use indexmap::IndexMap;
use proc_macro2::{Literal, TokenStream};
use quote::quote;
use serde_json::Value;
use std::fs;

pub fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/registry/1_21_9_synced_registries.json");
    println!("cargo:rerun-if-changed=../assets/registry/1_21_11_synced_registries.json");

    let process_version = |path: &str| -> TokenStream {
        let json_str = fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read {path}"));
        let mut data: IndexMap<String, IndexMap<String, Value>> =
            serde_json::from_str(&json_str).expect("Failed to parse JSON");

        // Inject "raw" chat type for vanilla parity
        if let Some(chat) = data.get_mut("minecraft:chat_type") {
            chat.insert("raw".to_owned(), serde_json::json!({
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
                        let mut bytes = Vec::new();
                        pumpkin_nbt::serializer::to_bytes_unnamed(entry_data, &mut bytes).unwrap();
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

    let v1_21_9_registries = process_version("../assets/registry/1_21_9_synced_registries.json");
    let v1_21_11_registries = process_version("../assets/registry/1_21_11_synced_registries.json");

    quote! {
        use pumpkin_util::resource_location::ResourceLocation;
        use pumpkin_util::version::MinecraftVersion;

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

        pub static REGISTRIES_1_21_9: &[StaticRegistry] = #v1_21_9_registries;
        pub static REGISTRIES_1_21_11: &[StaticRegistry] = #v1_21_11_registries;

        impl Registry {
            pub fn get_synced(version: MinecraftVersion) -> Vec<Self> {
                let static_regs = match version {
                    MinecraftVersion::V_1_21_9 => REGISTRIES_1_21_9,
                    _ => REGISTRIES_1_21_11,
                };

                static_regs.iter().map(|static_reg| {
                    let registry_id = if static_reg.registry_id.contains(':') {
                        ResourceLocation::from(static_reg.registry_id)
                    } else {
                        ResourceLocation::vanilla(static_reg.registry_id)
                    };

                    let registry_entries = static_reg.entries.iter().map(|entry| {
                        let entry_id =
                            ResourceLocation::vanilla(entry.name);

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
