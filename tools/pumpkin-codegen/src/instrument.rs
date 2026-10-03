use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

#[derive(Deserialize)]
struct InstrumentJson {
    description: Option<DescriptionJson>,
    range: f32,
    sound_event: String,
    use_duration: f32,
}

#[derive(Deserialize)]
struct DescriptionJson {
    translate: Option<String>,
}

pub fn build() -> TokenStream {
    let dir = Path::new("../../assets/datapack/data/minecraft/instrument");

    let mut instruments: BTreeMap<String, InstrumentJson> = BTreeMap::new();

    let mut entries = fs::read_dir(dir)
        .expect("read instrument dir")
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if !path.extension().is_some_and(|ext| ext == "json") {
            continue;
        }

        let stem = path.file_stem().unwrap().to_string_lossy().to_string();
        let content = fs::read_to_string(&path).expect("read instrument file");
        let json: InstrumentJson = serde_json::from_str(&content).expect("parse instrument JSON");

        instruments.insert(stem, json);
    }

    let mut enum_variants = Vec::new();
    let mut from_name_arms = Vec::new();
    let mut to_name_arms = Vec::new();
    let mut asset_id_arms = Vec::new();
    let mut sound_arms = Vec::new();
    let mut duration_sec_arms = Vec::new();
    let mut duration_ticks_arms = Vec::new();
    let mut range_arms = Vec::new();
    let mut translation_key_arms = Vec::new();
    let mut all_variants = Vec::new();

    for (name, json) in &instruments {
        let variant_ident = format_ident!("{}", name.to_pascal_case());
        let namespaced = format!("minecraft:{name}");
        let description_key = json
            .description
            .as_ref()
            .and_then(|d| d.translate.clone())
            .unwrap_or_else(|| format!("instrument.minecraft.{name}"));
        let range = json.range;
        let use_duration_sec = json.use_duration;
        let use_duration_ticks = (json.use_duration * 20.0).round() as u32;

        // Map sound_event string (e.g. "minecraft:item.goat_horn.sound.0") to Sound identifier
        let sound_name = json
            .sound_event
            .strip_prefix("minecraft:")
            .unwrap_or(&json.sound_event);
        let sound_ident = format_ident!("{}", sound_name.replace('.', "_").to_pascal_case());

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
        sound_arms.push(quote! {
            Self::#variant_ident => crate::sound::Sound::#sound_ident
        });
        duration_sec_arms.push(quote! {
            Self::#variant_ident => #use_duration_sec
        });
        duration_ticks_arms.push(quote! {
            Self::#variant_ident => #use_duration_ticks
        });
        range_arms.push(quote! {
            Self::#variant_ident => #range
        });
        translation_key_arms.push(quote! {
            Self::#variant_ident => #description_key
        });
        all_variants.push(quote! { Self::#variant_ident });
    }

    quote! {
        /* This file is generated. Do not edit manually. */

        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        #[repr(u32)]
        pub enum Instrument {
            #(#enum_variants,)*
        }

        impl Instrument {
            #[doc = "Returns the instrument from a resource name (bare or namespaced)."]
            #[must_use]
            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    #(#from_name_arms,)*
                    _ => None,
                }
            }

            #[doc = "Returns the numeric ID of the instrument in the synced registry."]
            #[must_use]
            pub const fn id(&self) -> u32 {
                *self as u32
            }

            #[doc = "Returns the bare string name of the instrument."]
            #[must_use]
            pub const fn to_name(&self) -> &'static str {
                match self {
                    #(#to_name_arms,)*
                }
            }

            #[doc = "Returns the fully-qualified asset id of the instrument."]
            #[must_use]
            pub const fn asset_id(&self) -> &'static str {
                match self {
                    #(#asset_id_arms,)*
                }
            }

            #[doc = "Returns the sound event played by this instrument."]
            #[must_use]
            pub const fn sound(&self) -> crate::sound::Sound {
                match self {
                    #(#sound_arms,)*
                }
            }

            #[doc = "Returns the use duration of the instrument in seconds."]
            #[must_use]
            pub const fn use_duration_seconds(&self) -> f32 {
                match self {
                    #(#duration_sec_arms,)*
                }
            }

            #[doc = "Returns the use duration of the instrument in game ticks."]
            #[must_use]
            pub const fn use_duration_ticks(&self) -> u32 {
                match self {
                    #(#duration_ticks_arms,)*
                }
            }

            #[doc = "Returns the audibility range of the instrument in blocks."]
            #[must_use]
            pub const fn range(&self) -> f32 {
                match self {
                    #(#range_arms,)*
                }
            }

            #[doc = "Returns the translation key describing this instrument."]
            #[must_use]
            pub const fn translation_key(&self) -> &'static str {
                match self {
                    #(#translation_key_arms,)*
                }
            }

            #[doc = "Returns all vanilla instrument variants."]
            #[must_use]
            pub const fn all() -> &'static [Self] {
                &[
                    #(#all_variants,)*
                ]
            }
        }
    }
}
