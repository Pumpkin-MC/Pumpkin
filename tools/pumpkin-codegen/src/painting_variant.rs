use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

#[derive(Deserialize)]
struct PaintingJson {
    asset_id: String,
    height: u32,
    width: u32,
    title: Option<TextJson>,
    author: Option<TextJson>,
}

#[derive(Deserialize)]
struct TextJson {
    translate: Option<String>,
}

#[derive(Deserialize)]
struct TagJson {
    values: Vec<String>,
}

pub fn build() -> TokenStream {
    let dir = Path::new("../../assets/datapack/data/minecraft/painting_variant");
    let tag_file =
        Path::new("../../assets/datapack/data/minecraft/tags/painting_variant/placeable.json");

    let placeable_set: Vec<String> = if tag_file.exists() {
        let content = fs::read_to_string(tag_file).expect("read placeable.json");
        let tag: TagJson = serde_json::from_str(&content).expect("parse placeable.json");
        tag.values
    } else {
        Vec::new()
    };

    let mut paintings: BTreeMap<String, (PaintingJson, bool)> = BTreeMap::new();

    let mut entries = fs::read_dir(dir)
        .expect("read painting_variant dir")
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if !path.extension().is_some_and(|ext| ext == "json") {
            continue;
        }

        let stem = path.file_stem().unwrap().to_string_lossy().to_string();
        let content = fs::read_to_string(&path).expect("read painting file");
        let json: PaintingJson = serde_json::from_str(&content).expect("parse painting JSON");

        let is_placeable = placeable_set
            .iter()
            .any(|v| v == &format!("minecraft:{stem}") || v == &stem);

        paintings.insert(stem, (json, is_placeable));
    }

    let mut enum_variants = Vec::new();
    let mut from_name_arms = Vec::new();
    let mut to_name_arms = Vec::new();
    let mut asset_id_arms = Vec::new();
    let mut width_arms = Vec::new();
    let mut height_arms = Vec::new();
    let mut title_arms = Vec::new();
    let mut author_arms = Vec::new();
    let mut is_placeable_arms = Vec::new();
    let mut all_idents = Vec::new();
    let mut placeable_idents = Vec::new();

    for (stem, (json, is_placeable)) in &paintings {
        let variant_name = format_ident!("{}", stem.to_pascal_case());
        let asset_id = &json.asset_id;
        let width = json.width;
        let height = json.height;
        let title = json
            .title
            .as_ref()
            .and_then(|t| t.translate.as_deref())
            .unwrap_or("");
        let author_tokens = match json.author.as_ref().and_then(|a| a.translate.as_deref()) {
            Some(author) => quote! { Some(#author) },
            None => quote! { None },
        };

        enum_variants.push(quote! { #variant_name, });
        from_name_arms.push(quote! {
            #asset_id | #stem => Some(Self::#variant_name),
        });
        to_name_arms.push(quote! {
            Self::#variant_name => #stem,
        });
        asset_id_arms.push(quote! {
            Self::#variant_name => #asset_id,
        });
        width_arms.push(quote! {
            Self::#variant_name => #width,
        });
        height_arms.push(quote! {
            Self::#variant_name => #height,
        });
        title_arms.push(quote! {
            Self::#variant_name => #title,
        });
        author_arms.push(quote! {
            Self::#variant_name => #author_tokens,
        });
        is_placeable_arms.push(quote! {
            Self::#variant_name => #is_placeable,
        });

        all_idents.push(quote! { Self::#variant_name });
        if *is_placeable {
            placeable_idents.push(quote! { Self::#variant_name });
        }
    }

    quote! {
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        #[repr(u32)]
        pub enum PaintingVariant {
            #( #enum_variants )*
        }

        impl PaintingVariant {
            #[doc = "Returns the painting variant from a resource name (bare or namespaced)."]
            #[must_use]
            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    #( #from_name_arms )*
                    _ => None,
                }
            }

            #[doc = "Returns the numeric ID of the painting variant in the synced registry."]
            #[must_use]
            pub const fn id(&self) -> u32 {
                *self as u32
            }

            #[doc = "Returns the bare string name of the painting variant."]
            #[must_use]
            pub const fn to_name(&self) -> &'static str {
                match self {
                    #( #to_name_arms )*
                }
            }

            #[doc = "Returns the fully-qualified asset id of the painting variant."]
            #[must_use]
            pub const fn asset_id(&self) -> &'static str {
                match self {
                    #( #asset_id_arms )*
                }
            }

            #[doc = "Returns the width of the painting in blocks."]
            #[must_use]
            pub const fn width(&self) -> u32 {
                match self {
                    #( #width_arms )*
                }
            }

            #[doc = "Returns the height of the painting in blocks."]
            #[must_use]
            pub const fn height(&self) -> u32 {
                match self {
                    #( #height_arms )*
                }
            }

            #[doc = "Returns the translation key for the title of the painting."]
            #[must_use]
            pub const fn title(&self) -> &'static str {
                match self {
                    #( #title_arms )*
                }
            }

            #[doc = "Returns the translation key for the author of the painting, if any."]
            #[must_use]
            pub const fn author(&self) -> Option<&'static str> {
                match self {
                    #( #author_arms )*
                }
            }

            #[doc = "Returns whether this painting variant is placeable in survival mode (in `#minecraft:placeable`)."]
            #[must_use]
            pub const fn is_placeable(&self) -> bool {
                match self {
                    #( #is_placeable_arms )*
                }
            }

            #[doc = "Returns all painting variants."]
            #[must_use]
            pub const fn all() -> &'static [Self] {
                &[
                    #( #all_idents ),*
                ]
            }

            #[doc = "Returns all placeable painting variants."]
            #[must_use]
            pub const fn all_placeable() -> &'static [Self] {
                &[
                    #( #placeable_idents ),*
                ]
            }
        }
    }
}
