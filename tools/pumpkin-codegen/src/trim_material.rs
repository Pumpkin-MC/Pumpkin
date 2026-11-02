use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

#[derive(Deserialize)]
struct TrimMaterialJson {
    description: TrimDescriptionJson,
    palette_id: String,
}

#[derive(Deserialize)]
struct TrimDescriptionJson {
    color: Option<String>,
    translate: Option<String>,
}

pub fn build() -> TokenStream {
    let dir = Path::new("../../assets/datapack/data/minecraft/trim_material");

    let mut materials: BTreeMap<String, TrimMaterialJson> = BTreeMap::new();

    let mut entries = fs::read_dir(dir)
        .expect("read trim_material dir")
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if !path.extension().is_some_and(|ext| ext == "json") {
            continue;
        }

        let stem = path.file_stem().unwrap().to_string_lossy().to_string();
        let content = fs::read_to_string(&path).expect("read trim_material file");
        let json: TrimMaterialJson =
            serde_json::from_str(&content).expect("parse trim_material JSON");

        materials.insert(stem, json);
    }

    let mut enum_variants = Vec::new();
    let mut from_name_arms = Vec::new();
    let mut to_name_arms = Vec::new();
    let mut asset_id_arms = Vec::new();
    let mut palette_id_arms = Vec::new();
    let mut color_arms = Vec::new();
    let mut translation_key_arms = Vec::new();
    let mut all_variants = Vec::new();

    for (name, json) in &materials {
        let variant_ident = format_ident!("{}", name.to_pascal_case());
        let namespaced = format!("minecraft:{name}");
        let palette_id = &json.palette_id;
        let color = json.description.color.as_deref().unwrap_or("");
        let translation_key = json.description.translate.as_deref().unwrap_or("");

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
        palette_id_arms.push(quote! {
            Self::#variant_ident => #palette_id
        });
        color_arms.push(quote! {
            Self::#variant_ident => #color
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
        pub enum TrimMaterial {
            #(#enum_variants,)*
        }

        impl TrimMaterial {
            #[doc = "Returns the trim material from a resource name (bare or namespaced)."]
            #[must_use]
            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    #(#from_name_arms,)*
                    _ => None,
                }
            }

            #[doc = "Returns the numeric ID of the trim material in the synced registry."]
            #[must_use]
            pub const fn id(&self) -> u32 {
                *self as u32
            }

            #[doc = "Returns the bare string name of the trim material."]
            #[must_use]
            pub const fn to_name(&self) -> &'static str {
                match self {
                    #(#to_name_arms,)*
                }
            }

            #[doc = "Returns the fully-qualified asset id of the trim material."]
            #[must_use]
            pub const fn asset_id(&self) -> &'static str {
                match self {
                    #(#asset_id_arms,)*
                }
            }

            #[doc = "Returns the palette id for texture styling."]
            #[must_use]
            pub const fn palette_id(&self) -> &'static str {
                match self {
                    #(#palette_id_arms,)*
                }
            }

            #[doc = "Returns the hex color code for this material."]
            #[must_use]
            pub const fn color(&self) -> &'static str {
                match self {
                    #(#color_arms,)*
                }
            }

            #[doc = "Returns the translation key describing this trim material."]
            #[must_use]
            pub const fn translation_key(&self) -> &'static str {
                match self {
                    #(#translation_key_arms,)*
                }
            }

            #[doc = "Returns all vanilla trim materials."]
            #[must_use]
            pub const fn all() -> &'static [Self] {
                &[
                    #(#all_variants,)*
                ]
            }
        }
    }
}
