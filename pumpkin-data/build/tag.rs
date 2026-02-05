use std::{
    collections::{BTreeMap, HashSet},
    fs,
};

use crate::block::BlockAssets;
use crate::enchantments::Enchantment;
use crate::entity_type::EntityType;
use crate::fluid::Fluid;
use crate::item::Item;
use crate::{biome::Biome, version::MinecraftVersion};
use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};

// --- EnumCreator with from_string support ---
pub struct EnumCreator {
    pub name: String,
    pub values: Vec<String>,
}

impl ToTokens for EnumCreator {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = format_ident!("{}", self.name.to_pascal_case());

        let variants = self.values.iter().map(|v| {
            let variant_name = format_ident!("{}", v.to_pascal_case());
            quote! { #variant_name }
        });

        let from_string_arms = self.values.iter().map(|v| {
            let variant_name = format_ident!("{}", v.to_pascal_case());
            quote! { #v => Some(Self::#variant_name) }
        });

        let to_string_arms = self.values.iter().map(|v| {
            let variant_name = format_ident!("{}", v.to_pascal_case());
            quote! { Self::#variant_name => #v }
        });

        tokens.extend(quote! {
            #[derive(Eq, PartialEq, Hash, Debug, Clone, Copy)]
            pub enum #name {
                #(#variants),*
            }

            impl #name {
                pub fn from_string(s: &str) -> Option<Self> {
                    match s {
                        #(#from_string_arms,)*
                        _ => None,
                    }
                }

                pub fn identifier_string(&self) -> &str {
                    match self {
                        #(#to_string_arms),*
                    }
                }
            }
        });
    }
}

const LATEST_VERSION: MinecraftVersion = MinecraftVersion::V_1_21_11;

pub(crate) fn build() -> TokenStream {
    // --- Rerun Triggers ---
    println!("cargo:rerun-if-changed=../assets/blocks.json");
    println!("cargo:rerun-if-changed=../assets/items.json");
    println!("cargo:rerun-if-changed=../assets/biome.json");
    println!("cargo:rerun-if-changed=../assets/fluids.json");
    println!("cargo:rerun-if-changed=../assets/entities.json");

    // Watch specific tag versions
    let assets = [
        (MinecraftVersion::V_1_21_4, "1_21_4_tags.json"),
        (MinecraftVersion::V_1_21_5, "1_21_5_tags.json"),
        (MinecraftVersion::V_1_21_6, "1_21_6_tags.json"),
        (MinecraftVersion::V_1_21_9, "1_21_9_tags.json"),
        (MinecraftVersion::V_1_21_11, "1_21_11_tags.json"),
    ];

    // --- Load Global Assets ---
    let blocks_assets: BlockAssets =
        serde_json::from_str(&fs::read_to_string("../assets/blocks.json").unwrap())
            .expect("Failed to parse blocks.json");
    let items: BTreeMap<String, Item> =
        serde_json::from_str(&fs::read_to_string("../assets/items.json").unwrap())
            .expect("Failed to parse items.json");
    let biomes: BTreeMap<String, Biome> =
        serde_json::from_str(&fs::read_to_string("../assets/biome.json").unwrap())
            .expect("Failed to parse biome.json");
    let fluids: Vec<Fluid> =
        serde_json::from_str(&fs::read_to_string("../assets/fluids.json").unwrap())
            .expect("Failed to parse fluids.json");
    let enchantments: BTreeMap<String, Enchantment> =
        serde_json::from_str(&fs::read_to_string("../assets/enchantments.json").unwrap())
            .expect("Failed to parse enchantments.json");
    let entities: BTreeMap<String, EntityType> =
        serde_json::from_str(&fs::read_to_string("../assets/entities.json").unwrap())
            .expect("Failed to parse entities.json");

    let block_id_map: BTreeMap<String, u16> = blocks_assets
        .blocks
        .iter()
        .map(|b| (b.name.clone(), b.id))
        .collect();
    let fluid_id_map: BTreeMap<String, u16> =
        fluids.iter().map(|f| (f.name.clone(), f.id)).collect();

    let mut all_registry_keys = HashSet::new();
    let mut latest_modules = Vec::new();
    let mut legacy_modules = Vec::new();

    let mut match_get_map = Vec::new();

    for (ver, file) in assets {
        let file_path = format!("../assets/tags/{file}");
        println!("cargo:rerun-if-changed={file_path}");

        let tags: BTreeMap<String, BTreeMap<String, Vec<String>>> =
            serde_json::from_str(&fs::read_to_string(&file_path).unwrap()).unwrap();

        let is_latest = ver == LATEST_VERSION;
        let mut tag_dicts = Vec::new();
        let mut match_local_map = Vec::new();

        for (key, tag_map) in tags {
            all_registry_keys.insert(key.clone());
            let key_pascal = format_ident!("{}", key.to_pascal_case());
            let dict_name = format_ident!("{}_TAGS", key.to_pascal_case().to_uppercase());

            let mut tag_entries = Vec::new();
            let mut tag_map_entries = Vec::new();
            let tag_type_path = if is_latest {
                quote!(super::Tag)
            } else {
                quote!(super::super::Tag)
            };

            for (tag_name, values) in tag_map {
                let ids: Vec<u16> = values
                    .iter()
                    .filter_map(|v| match key.as_str() {
                        "worldgen/biome" => biomes.get(v).map(|b| u16::from(b.id)),
                        "fluid" => fluid_id_map.get(v).copied(),
                        "item" => items.get(v).map(|i| i.id),
                        "block" => block_id_map.get(v).copied(),
                        "enchantment" => enchantments
                            .get(&format!("minecraft:{v}"))
                            .map(|e| u16::from(e.id)),
                        "entity_type" => entities.get(v).map(|e| e.id),
                        _ => None,
                    })
                    .collect();

                let tag_const_name =
                    format_ident!("{}", tag_name.replace([':', '/'], "_").to_uppercase());

                tag_entries.push(quote! {
                    pub const #tag_const_name: #tag_type_path = (&[#(#values),*], &[#(#ids),*]);
                });
                tag_map_entries.push(quote! { #tag_name => &#key_pascal::#tag_const_name });
            }

            let tag_type_path = if is_latest {
                quote!(Tag)
            } else {
                quote!(super::Tag)
            };

            tag_dicts.push(quote! {
                #[allow(non_snake_case)]
                pub mod #key_pascal {
                    #(#tag_entries)* }
                pub(crate) static #dict_name: phf::Map<&'static str, &'static #tag_type_path> = phf::phf_map! {
                    #(#tag_map_entries),* };
            });

            match_local_map.push(quote! { RegistryKey::#key_pascal => Some(&#dict_name) });
        }

        if is_latest {
            latest_modules.push(quote! {
                #(#tag_dicts)*
                #[allow(unreachable_patterns)]
                pub fn get_latest_map(key: RegistryKey) -> Option<&'static phf::Map<&'static str, &'static Tag>> {
                    match key { #(#match_local_map,)* _ => None }
                }
            });
            match_get_map.push(quote! { #LATEST_VERSION => get_latest_map(tag_category) });
        } else {
            let mod_name = format_ident!("tags_{}", ver.to_field_ident());
            legacy_modules.push(quote! {
                mod #mod_name {
                    use super::RegistryKey;
                    #(#tag_dicts)*
                    pub fn get_map(key: RegistryKey) -> Option<&'static phf::Map<&'static str, &'static super::Tag>> {
                        match key { #(#match_local_map,)* _ => None }
                    }
                }
            });
            match_get_map.push(quote! { #ver => #mod_name::get_map(tag_category) });
        }
    }

    // --- Generate RegistryKey Enum ---
    let registry_key_enum = EnumCreator {
        name: "RegistryKey".to_string(),
        values: all_registry_keys.into_iter().collect(),
    }
    .to_token_stream();

    quote! {
        use pumpkin_util::version::MinecraftVersion;

        pub type Tag = (&'static [&'static str], &'static [u16]);

        #registry_key_enum

        // Latest tags are exported directly here
        #(#latest_modules)*

        // Legacy tags are hidden in their own module
        #(#legacy_modules)*

        pub fn get_tag_values(tag_category: RegistryKey, tag: &str) -> Option<&'static [&'static str]> {
            get_latest_map(tag_category).and_then(|m| m.get(tag)).map(|t| t.0)
        }

        pub fn get_tag_ids(tag_category: RegistryKey, tag: &str) -> Option<&'static [u16]> {
            get_latest_map(tag_category).and_then(|m| m.get(tag)).map(|t| t.1)
        }

        pub fn get_registry_key_tags(version: MinecraftVersion, tag_category: RegistryKey) -> Option<&'static phf::Map<&'static str, &'static Tag>> {
            match version {
                #(#match_get_map),*,
                _ => get_latest_map(tag_category)
            }
        }

        pub trait Taggable {
            fn tag_key() -> RegistryKey;
            fn registry_key(&self) -> &str;
            fn registry_id(&self) -> u16;

           fn is_tagged_with(&self, tag: &str) -> Option<bool> {
                let tag = tag.strip_prefix("#").unwrap_or(tag);
                let items = get_tag_ids(Self::tag_key(), tag)?;
                Some(items.contains(&self.registry_id()))
            }

            fn has_tag(&self, tag: &'static Tag) -> bool {
                tag.1.contains(&self.registry_id())
            }

            fn get_tag_values(tag: &str) -> Option<&'static [&'static str]> {
                let tag = tag.strip_prefix("#").unwrap_or(tag);
                get_tag_values(Self::tag_key(), tag)
            }
        }
    }
}
