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
        }
    }
}
