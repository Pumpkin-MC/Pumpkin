use std::{collections::BTreeMap, fs};

use crate::biome::Biome;
use crate::block::BlockAssets;
use crate::enchantments::Enchantment;
use crate::entity_type::EntityType;
use crate::fluid::Fluid;
use crate::item::Item;
use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};

pub struct EnumCreator {
    pub name: String,
    pub value: Vec<String>,
}

impl ToTokens for EnumCreator {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = format_ident!("{}", self.name.to_pascal_case());
        let values = self
            .value
            .iter()
            .map(|value| {
                let name = format_ident!("{}", value.to_pascal_case());
                name
            })
            .collect::<Vec<_>>();
        tokens.extend(quote! {
            pub enum #name {
                #(#values),*
            }
        });
    }
}
pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/tags.json");
    println!("cargo:rerun-if-changed=../assets/blocks.json");
    println!("cargo:rerun-if-changed=../assets/items.json");
    println!("cargo:rerun-if-changed=../assets/biome.json");
    println!("cargo:rerun-if-changed=../assets/fluids.json");
    println!("cargo:rerun-if-changed=../assets/entities.json");

    let tags: BTreeMap<String, BTreeMap<String, Vec<String>>> =
        serde_json::from_str(&fs::read_to_string("../assets/tags.json").unwrap())
            .expect("Failed to parse tags.json");

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
        match serde_json::from_str(&fs::read_to_string("../assets/fluids.json").unwrap()) {
            Ok(fluids) => fluids,
            Err(e) => panic!("Failed to parse fluids.json: {e}"),
        };

    let enchantments: BTreeMap<String, Enchantment> =
        serde_json::from_str(&fs::read_to_string("../assets/enchantments.json").unwrap())
            .expect("Failed to parse enchantments.json");

    let entities: BTreeMap<String, EntityType> =
        serde_json::from_str(&fs::read_to_string("../assets/entities.json").unwrap())
            .expect("Failed to parse entities.json");

    let registry_key_enum = EnumCreator {
        name: "RegistryKey".to_string(),
        value: tags.keys().map(|key| key.to_string()).collect(),
    }
    .to_token_stream();

    let block_id_map: BTreeMap<String, u16> = blocks_assets
        .blocks
        .iter()
        .map(|b| (b.name.clone(), b.id))
        .collect();
    let fluid_id_map: BTreeMap<String, u16> =
        fluids.iter().map(|f| (f.name.clone(), f.id)).collect();

    // Generate tag arrays for each registry key
    let mut tag_dicts = Vec::new();
    let mut match_arms_value = Vec::new();
    let mut match_arms_id = Vec::new();
    let mut match_arms_tags_all = Vec::new();
    let mut tag_identifiers = Vec::new();

    for (key, tag_map) in tags {
        let key_pascal = format_ident!("{}", key.to_pascal_case());
        let dict_name = format_ident!("{}_TAGS", key.to_pascal_case().to_uppercase());

        let mut tag_entries = Vec::new();
        let mut tag_map_entries = Vec::new();

        for (tag_name, values) in tag_map {
            let ids: Vec<u16> = values
                .iter()
                .filter_map(|v| match key.as_str() {
                    "worldgen/biome" => biomes.get(v).map(|b| b.id as u16),
                    "fluid" => fluid_id_map.get(v).copied(),
                    "item" => items.get(v).map(|i| i.id),
                    "block" => block_id_map.get(v).copied(),
                    "enchantment" => enchantments
                        .get(&format!("minecraft:{}", v))
                        .map(|e| e.id as u16),
                    "entity_type" => entities.get(v).map(|e| e.id),
                    _ => None,
                })
                .collect();
            let tag_const_name =
                format_ident!("{}", tag_name.replace([':', '/'], "_").to_uppercase());

            tag_entries.push(quote! {
                pub const #tag_const_name: Tag = (&[#(#values),*], &[#(#ids),*]);
            });

            tag_map_entries.push(quote! {
                #tag_name => &#key_pascal::#tag_const_name
            });
        }

        tag_dicts.push(quote! {
            #[allow(non_snake_case)]
            pub mod #key_pascal {
                use super::Tag;
                #(#tag_entries)*
            }
            static #dict_name: phf::Map<&'static str, &'static Tag> = phf::phf_map! {
                #(#tag_map_entries),*
            };
        });

        // Add match arm for this registry key
        match_arms_value.push(quote! {
            RegistryKey::#key_pascal => {
                #dict_name.get(tag).map(|i| i.0)
            }
        });

        match_arms_id.push(quote! {
            RegistryKey::#key_pascal => {
                #dict_name.get(tag).map(|i| i.1 )
            }
        });

        match_arms_tags_all.push(quote! {
            RegistryKey::#key_pascal => {
                &#dict_name
            }
        });

        tag_identifiers.push(quote! {
            Self::#key_pascal => #key
        });
    }

    quote! {
        #[derive(Eq, PartialEq, Hash, Debug)]
        #registry_key_enum

        impl RegistryKey {
            // IDK why the linter is saying this isn't used
            #[allow(dead_code)]
            pub fn identifier_string(&self) -> &str {
                match self {
                    #(#tag_identifiers),*
                }
            }
        }

        pub type Tag = (&'static [&'static str], &'static [u16]);

        #(#tag_dicts)*

        pub fn get_tag_values(tag_category: RegistryKey, tag: &str) -> Option<&'static [&'static str]> {
            match tag_category {
                #(#match_arms_value),*
            }
        }

        pub fn get_tag_ids(tag_category: RegistryKey, tag: &str) -> Option<&'static [u16]> {
            match tag_category {
                #(#match_arms_id),*
            }
        }

        pub fn get_registry_key_tags(tag_category: &RegistryKey) -> &phf::Map<&'static str, &'static Tag> {
            match tag_category {
                #(#match_arms_tags_all),*
            }
        }

        pub trait Taggable {
            fn tag_key() -> RegistryKey;
            fn registry_key(&self) -> &str;
            fn registry_id(&self) -> u16;

            /// Returns `None` if the tag does not exist.
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
