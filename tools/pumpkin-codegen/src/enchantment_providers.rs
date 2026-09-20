use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

/// One `enchantment_provider` definition, as the vanilla datapack writes it.
///
/// `minecraft:by_cost` is absent on purpose: no vanilla provider uses it, so there is no
/// data to check an implementation of it against.
#[derive(Deserialize)]
#[serde(tag = "type")]
enum Provider {
    /// A fixed enchantment at a fixed level.
    #[serde(rename = "minecraft:single")]
    Single {
        enchantment: String,
        /// Vanilla types this as an `IntProvider`, but every provider it ships uses a
        /// plain number, so that is what is read here.
        level: i32,
    },
    /// A draw from a tag, at a cost that widens with the regional difficulty.
    #[serde(rename = "minecraft:by_cost_with_difficulty")]
    ByCostWithDifficulty {
        enchantments: String,
        min_cost: i32,
        max_cost_span: i32,
    },
}

/// `raid/vindicator_post_wave_5.json` -> `RAID_VINDICATOR_POST_WAVE_5`.
fn constant_name(relative_path: &Path) -> String {
    relative_path
        .with_extension("")
        .components()
        .map(|component| {
            component
                .as_os_str()
                .to_string_lossy()
                .to_shouty_snake_case()
        })
        .collect::<Vec<_>>()
        .join("_")
}

fn tag_ident(tag: &str) -> proc_macro2::Ident {
    format_ident!(
        "{}",
        tag.strip_prefix('#')
            .unwrap_or(tag)
            .replace([':', '/'], "_")
            .to_uppercase()
    )
}

fn collect(dir: &Path, root: &Path, out: &mut BTreeMap<String, Provider>) {
    let entries = fs::read_dir(dir).expect("Missing enchantment_provider directory");
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, root, out);
        } else if path.extension().is_some_and(|ext| ext == "json") {
            let relative = path.strip_prefix(root).expect("path is under root");
            let content = fs::read_to_string(&path).expect("Failed to read provider file");
            let provider: Provider =
                serde_json::from_str(&content).expect("Failed to parse provider JSON");
            out.insert(constant_name(relative), provider);
        }
    }
}

pub fn build() -> TokenStream {
    let root = Path::new("../../assets/datapack/data/minecraft/enchantment_provider");
    let mut providers = BTreeMap::new();
    collect(root, root, &mut providers);

    let constants = providers.iter().map(|(name, provider)| {
        let ident = format_ident!("{}", name);
        let value = match provider {
            Provider::Single { enchantment, level } => {
                let enchantment = format_ident!(
                    "{}",
                    enchantment
                        .strip_prefix("minecraft:")
                        .unwrap_or(enchantment)
                        .to_uppercase()
                );
                quote! {
                    Self::Single {
                        enchantment: &Enchantment::#enchantment,
                        level: #level,
                    }
                }
            }
            Provider::ByCostWithDifficulty {
                enchantments,
                min_cost,
                max_cost_span,
            } => {
                let tag = tag_ident(enchantments);
                quote! {
                    Self::ByCostWithDifficulty {
                        enchantments: &EnchantmentTag::#tag,
                        min_cost: #min_cost,
                        max_cost_span: #max_cost_span,
                    }
                }
            }
        };
        quote! { pub const #ident: Self = #value; }
    });

    quote! {
        use crate::Enchantment;
        use crate::tag::Enchantment as EnchantmentTag;
        use crate::tag::Tag;

        /// How something that is not a player at an enchanting table decides what to put on
        /// an item. Vanilla calls these enchantment providers and ships them as datapack
        /// entries; this is that data.
        #[derive(Clone, Copy)]
        pub enum EnchantmentProvider {
            /// One enchantment at one level, upgraded onto the item.
            Single {
                enchantment: &'static Enchantment,
                level: i32,
            },
            /// A draw from `enchantments` at a cost between `min_cost` and
            /// `min_cost + difficulty * max_cost_span`.
            ByCostWithDifficulty {
                enchantments: &'static Tag,
                min_cost: i32,
                max_cost_span: i32,
            },
        }

        impl EnchantmentProvider {
            #(#constants)*
        }
    }
}
