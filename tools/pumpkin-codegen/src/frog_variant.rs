use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

#[derive(Deserialize)]
struct FrogVariantJson {
    asset_id: String,
}

pub fn build() -> TokenStream {
    let dir = Path::new("../../assets/datapack/data/minecraft/frog_variant");

    let mut variants: BTreeMap<String, FrogVariantJson> = BTreeMap::new();

    let mut entries = fs::read_dir(dir)
        .expect("read frog_variant dir")
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if !path.extension().is_some_and(|ext| ext == "json") {
            continue;
        }

        let stem = path.file_stem().unwrap().to_string_lossy().to_string();
        let content = fs::read_to_string(&path).expect("read frog_variant file");
        let json: FrogVariantJson =
            serde_json::from_str(&content).expect("parse frog_variant JSON");

        variants.insert(stem, json);
    }

    let mut enum_variants = Vec::new();
    let mut from_name_arms = Vec::new();
    let mut to_name_arms = Vec::new();
    let mut asset_id_arms = Vec::new();
    let mut texture_arms = Vec::new();
    let mut all_variants = Vec::new();

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
    }

    quote! {
        /* This file is generated. Do not edit manually. */

        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
        #[repr(u32)]
        pub enum FrogVariant {
            Cold,
            #[default]
            Temperate,
            Warm,
        }

        impl FrogVariant {
            #[doc = "Returns the frog variant from a resource name (bare or namespaced)."]
            #[must_use]
            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    #(#from_name_arms,)*
                    _ => None,
                }
            }

            #[doc = "Returns the numeric ID of the frog variant in the synced registry."]
            #[must_use]
            pub const fn id(&self) -> u32 {
                *self as u32
            }

            #[doc = "Returns the frog variant from numeric ID."]
            #[must_use]
            pub const fn from_id(id: u32) -> Self {
                match id {
                    0 => Self::Cold,
                    2 => Self::Warm,
                    _ => Self::Temperate,
                }
            }

            #[doc = "Returns the bare string name of the frog variant."]
            #[must_use]
            pub const fn to_name(&self) -> &'static str {
                match self {
                    #(#to_name_arms,)*
                }
            }

            #[doc = "Returns the fully-qualified asset id of the frog variant."]
            #[must_use]
            pub const fn asset_id(&self) -> &'static str {
                match self {
                    #(#asset_id_arms,)*
                }
            }

            #[doc = "Returns the texture asset path for the frog variant."]
            #[must_use]
            pub const fn texture(&self) -> &'static str {
                match self {
                    #(#texture_arms,)*
                }
            }

            #[doc = "Returns all vanilla frog variants."]
            #[must_use]
            pub const fn all() -> &'static [Self] {
                &[
                    #(#all_variants,)*
                ]
            }
        }
    }
}
