use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

#[derive(Deserialize)]
struct TrimPatternJson {
    asset_id: String,
    #[serde(default)]
    decal: bool,
    description: TrimDescriptionJson,
}

#[derive(Deserialize)]
struct TrimDescriptionJson {
    translate: Option<String>,
}

pub fn build() -> TokenStream {
    let dir = Path::new("../../assets/datapack/data/minecraft/trim_pattern");

    let mut patterns: BTreeMap<String, TrimPatternJson> = BTreeMap::new();

    let mut entries = fs::read_dir(dir)
        .expect("read trim_pattern dir")
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if !path.extension().is_some_and(|ext| ext == "json") {
            continue;
        }

        let stem = path.file_stem().unwrap().to_string_lossy().to_string();
        let content = fs::read_to_string(&path).expect("read trim_pattern file");
        let json: TrimPatternJson =
            serde_json::from_str(&content).expect("parse trim_pattern JSON");

        patterns.insert(stem, json);
    }

    let mut enum_variants = Vec::new();
    let mut from_name_arms = Vec::new();
    let mut to_name_arms = Vec::new();
    let mut asset_id_arms = Vec::new();
    let mut decal_arms = Vec::new();
    let mut translation_key_arms = Vec::new();
    let mut all_variants = Vec::new();

    for (name, json) in &patterns {
        let variant_ident = format_ident!("{}", name.to_pascal_case());
        let namespaced = format!("minecraft:{name}");
        let asset_id = &json.asset_id;
        let decal = json.decal;
        let translation_key = json.description.translate.as_deref().unwrap_or("");

        enum_variants.push(quote! { #variant_ident });
        from_name_arms.push(quote! {
            #namespaced | #name => Some(Self::#variant_ident)
        });
        to_name_arms.push(quote! {
            Self::#variant_ident => #name
        });
        asset_id_arms.push(quote! {
            Self::#variant_ident => #asset_id
        });
        decal_arms.push(quote! {
            Self::#variant_ident => #decal
        });
        translation_key_arms.push(quote! {
            Self::#variant_ident => #translation_key
        });
        all_variants.push(quote! { Self::#variant_ident });
    }

    quote! {
        /* This file is generated. Do not edit manually. */

        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        #[repr(u32)]
        pub enum TrimPattern {
            #(#enum_variants,)*
        }

        impl TrimPattern {
            #[doc = "Returns the trim pattern from a resource name (bare or namespaced)."]
            #[must_use]
            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    #(#from_name_arms,)*
                    _ => None,
                }
            }

            #[doc = "Returns the numeric ID of the trim pattern in the synced registry."]
            #[must_use]
            pub const fn id(&self) -> u32 {
                *self as u32
            }

            #[doc = "Returns the bare string name of the trim pattern."]
            #[must_use]
            pub const fn to_name(&self) -> &'static str {
                match self {
                    #(#to_name_arms,)*
                }
            }

            #[doc = "Returns the fully-qualified asset id of the trim pattern."]
            #[must_use]
            pub const fn asset_id(&self) -> &'static str {
                match self {
                    #(#asset_id_arms,)*
                }
            }

            #[doc = "Returns whether this trim pattern is a decal."]
            #[must_use]
            pub const fn is_decal(&self) -> bool {
                match self {
                    #(#decal_arms,)*
                }
            }

            #[doc = "Returns the translation key describing this trim pattern."]
            #[must_use]
            pub const fn translation_key(&self) -> &'static str {
                match self {
                    #(#translation_key_arms,)*
                }
            }

            #[doc = "Returns all vanilla trim patterns."]
            #[must_use]
            pub const fn all() -> &'static [Self] {
                &[
                    #(#all_variants,)*
                ]
            }
        }
    }
}
