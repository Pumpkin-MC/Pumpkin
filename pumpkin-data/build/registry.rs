use indexmap::IndexMap;
use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};
use serde_json::Value;
use std::fs;

use crate::version::MinecraftVersion;

const LATEST_VERSION: MinecraftVersion = MinecraftVersion::V_1_21_11;

pub(crate) fn build() -> TokenStream {
    let assets = [
        (MinecraftVersion::V_1_21_4, "1_21_4_synced_registries.json"),
        (MinecraftVersion::V_1_21_5, "1_21_5_synced_registries.json"),
        (MinecraftVersion::V_1_21_6, "1_21_6_synced_registries.json"),
        (MinecraftVersion::V_1_21_7, "1_21_7_synced_registries.json"),
        (MinecraftVersion::V_1_21_9, "1_21_9_synced_registries.json"),
        (
            MinecraftVersion::V_1_21_11,
            "1_21_11_synced_registries.json",
        ),
    ];

    let process_version = |path: &str| -> TokenStream {
        let json_str = fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read {path}"));
        let mut data: IndexMap<String, IndexMap<String, Value>> =
            serde_json::from_str(&json_str).expect("Failed to parse JSON");

        // Inject "raw" chat type for vanilla parity
        if let Some(chat) = data.get_mut("minecraft:chat_type") {
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

    let mut static_values = TokenStream::new();
    let mut match_arms = TokenStream::new();
    let mut latest_registry = None;

    for (ver, file) in assets {
        let path = format!("../assets/registry/{file}");
        println!("cargo:rerun-if-changed={path}");

        let registries = process_version(&path);

        let ident = format_ident!("REGISTRY_{ver:?}");

        static_values.extend(quote! {
            pub static #ident: &[StaticRegistry] = #registries;
        });

        match_arms.extend(quote! {
            #ver => #ident,
        });

        if ver == LATEST_VERSION {
            latest_registry = Some(ident);
        }
    }

    let latest_registry = latest_registry.unwrap();

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

        #static_values

        impl Registry {
            pub fn get_synced(version: MinecraftVersion) -> Vec<Self> {
                let static_regs = match version {
                    #match_arms
                    _ => #latest_registry,
                };

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
