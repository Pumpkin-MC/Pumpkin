use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

#[derive(Deserialize)]
struct CatVariantJson {
    asset_id: String,
    #[serde(default)]
    baby_asset_id: Option<String>,
    #[serde(default)]
    spawn_conditions: Vec<SpawnSelectorJson>,
}

#[derive(Deserialize)]
struct SpawnSelectorJson {
    priority: i32,
    #[serde(default)]
    condition: Option<SpawnConditionJson>,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum SpawnConditionJson {
    #[serde(rename = "minecraft:structure")]
    Structure { structures: String },
    #[serde(rename = "minecraft:moon_brightness")]
    MoonBrightness { range: DoubleRangeJson },
}

#[derive(Deserialize)]
#[serde(untagged)]
enum DoubleRangeJson {
    Exact(f64),
    Bounds {
        #[serde(default)]
        min: Option<f64>,
        #[serde(default)]
        max: Option<f64>,
    },
}

fn option_tokens(value: Option<f64>) -> TokenStream {
    match value {
        Some(v) => quote! { Some(#v) },
        None => quote! { None },
    }
}

fn condition_tokens(condition: &SpawnConditionJson) -> TokenStream {
    match condition {
        SpawnConditionJson::Structure { structures } => {
            let tag = structures.strip_prefix('#').unwrap_or_else(|| {
                panic!("cat_variant structure condition must be a tag: {structures}")
            });
            let tag_ident = format_ident!("{}", tag.replace([':', '/'], "_").to_uppercase());
            quote! { CatSpawnCondition::Structure(&crate::tag::WorldgenStructure::#tag_ident) }
        }
        SpawnConditionJson::MoonBrightness { range } => {
            let (min, max) = match *range {
                DoubleRangeJson::Exact(v) => (Some(v), Some(v)),
                DoubleRangeJson::Bounds { min, max } => (min, max),
            };
            let min = option_tokens(min);
            let max = option_tokens(max);
            quote! { CatSpawnCondition::MoonBrightness { min: #min, max: #max } }
        }
    }
}

pub fn build() -> TokenStream {
    let dir = Path::new("../../assets/datapack/data/minecraft/cat_variant");

    let mut variants: BTreeMap<String, CatVariantJson> = BTreeMap::new();

    let mut entries = fs::read_dir(dir)
        .expect("read cat_variant dir")
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if !path.extension().is_some_and(|ext| ext == "json") {
            continue;
        }

        let stem = path.file_stem().unwrap().to_string_lossy().to_string();
        let content = fs::read_to_string(&path).expect("read cat_variant file");
        let json: CatVariantJson = serde_json::from_str(&content).expect("parse cat_variant JSON");

        variants.insert(stem, json);
    }

    let mut enum_variants = Vec::new();
    let mut from_name_arms = Vec::new();
    let mut to_name_arms = Vec::new();
    let mut asset_id_arms = Vec::new();
    let mut texture_arms = Vec::new();
    let mut all_variants = Vec::new();
    let mut spawn_selector_arms = Vec::new();

    for (name, json) in &variants {
        let variant_ident = format_ident!("{}", name.to_pascal_case());
        let namespaced = format!("minecraft:{name}");
        let asset_id = &json.asset_id;

        enum_variants.push(quote! { #variant_ident });
        from_name_arms.push(quote! {
            #namespaced | #name => Some(Self::#variant_ident)
        });
        to_name_arms.push(quote! {
            Self::#variant_ident => #name
        });
        asset_id_arms.push(quote! {
            Self::#variant_ident => #namespaced
        });
        texture_arms.push(quote! {
            Self::#variant_ident => #asset_id
        });
        all_variants.push(quote! { Self::#variant_ident });
        let selectors = json.spawn_conditions.iter().map(|selector| {
            let priority = selector.priority;
            let condition = selector.condition.as_ref().map_or_else(
                || quote! { None },
                |c| {
                    let c = condition_tokens(c);
                    quote! { Some(#c) }
                },
            );
            quote! { CatSpawnSelector { priority: #priority, condition: #condition } }
        });
        spawn_selector_arms.push(quote! {
            Self::#variant_ident => &[#(#selectors,)*]
        });
    }

    quote! {
        /* This file is generated. Do not edit manually. */

        #[derive(Clone, Copy, PartialEq, Debug)]
        pub enum CatSpawnCondition {
            Structure(&'static crate::tag::Tag),
            MoonBrightness { min: Option<f64>, max: Option<f64> },
        }

        #[derive(Clone, Copy, PartialEq, Debug)]
        pub struct CatSpawnSelector {
            pub priority: i32,
            pub condition: Option<CatSpawnCondition>,
        }

        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        #[repr(u8)]
        pub enum CatVariant {
            #(#enum_variants,)*
        }

        impl CatVariant {
            #[doc = "Returns the cat variant from a resource name (bare or namespaced)."]
            #[must_use]
            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    #(#from_name_arms,)*
                    _ => None,
                }
            }

            #[doc = "Returns the numeric ID of the cat variant in the synced registry."]
            #[must_use]
            pub const fn id(&self) -> u8 {
                *self as u8
            }

            #[doc = "Returns the bare string name of the cat variant."]
            #[must_use]
            pub const fn to_name(&self) -> &'static str {
                match self {
                    #(#to_name_arms,)*
                }
            }

            #[doc = "Returns the fully-qualified asset id of the cat variant."]
            #[must_use]
            pub const fn asset_id(&self) -> &'static str {
                match self {
                    #(#asset_id_arms,)*
                }
            }

            #[doc = "Returns the texture asset path for the cat variant."]
            #[must_use]
            pub const fn texture(&self) -> &'static str {
                match self {
                    #(#texture_arms,)*
                }
            }

            #[doc = "Returns all vanilla cat variants."]
            #[must_use]
            pub const fn all() -> &'static [Self] {
                &[
                    #(#all_variants,)*
                ]
            }

            #[doc = "Returns the spawn selectors of the cat variant, from its `spawn_conditions`."]
            #[must_use]
            pub const fn spawn_selectors(&self) -> &'static [CatSpawnSelector] {
                match self {
                    #(#spawn_selector_arms,)*
                }
            }
        }
    }
}
