use std::fs;

use heck::{ToPascalCase, ToShoutySnakeCase};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

#[derive(Deserialize)]
struct DyeColorEntry {
    id: u8,
    name: String,
    map_color_id: u8,
    terracotta_color_id: u8,
    texture_diffuse_color: u32,
    firework_color: u32,
    text_color: u32,
}

pub fn build() -> TokenStream {
    let dye_colors: Vec<DyeColorEntry> =
        serde_json::from_str(&fs::read_to_string("../../assets/dye_colors.json").unwrap())
            .expect("Failed to parse dye_colors.json");

    let mut variants = TokenStream::new();
    let mut id_match_arms = TokenStream::new();
    let mut name_match_arms = TokenStream::new();
    let mut to_id_arms = TokenStream::new();
    let mut to_name_arms = TokenStream::new();
    let mut map_color_id_arms = TokenStream::new();
    let mut terracotta_color_id_arms = TokenStream::new();
    let mut texture_diffuse_color_arms = TokenStream::new();
    let mut firework_color_arms = TokenStream::new();
    let mut text_color_arms = TokenStream::new();

    for entry in &dye_colors {
        let id = entry.id;
        let name_str = &entry.name;
        let variant_ident = format_ident!("{}", name_str.to_pascal_case());
        let map_color_id = entry.map_color_id;
        let terracotta_color_id = entry.terracotta_color_id;
        let texture_diffuse_color = entry.texture_diffuse_color;
        let firework_color = entry.firework_color;
        let text_color = entry.text_color;

        let is_default = if name_str == "black" {
            quote! { #[default] }
        } else {
            quote! {}
        };

        variants.extend(quote! {
            #is_default
            #variant_ident,
        });

        id_match_arms.extend(quote! {
            #id => Some(Self::#variant_ident),
        });

        name_match_arms.extend(quote! {
            #name_str => Some(Self::#variant_ident),
        });

        to_id_arms.extend(quote! {
            Self::#variant_ident => #id,
        });

        to_name_arms.extend(quote! {
            Self::#variant_ident => #name_str,
        });

        map_color_id_arms.extend(quote! {
            Self::#variant_ident => #map_color_id,
        });

        terracotta_color_id_arms.extend(quote! {
            Self::#variant_ident => #terracotta_color_id,
        });

        texture_diffuse_color_arms.extend(quote! {
            Self::#variant_ident => #texture_diffuse_color,
        });

        firework_color_arms.extend(quote! {
            Self::#variant_ident => #firework_color,
        });

        text_color_arms.extend(quote! {
            Self::#variant_ident => #text_color,
        });
    }

    quote! {
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
        pub enum DyeColor {
            #variants
        }

        impl DyeColor {
            #[must_use]
            pub const fn id(&self) -> u8 {
                match self {
                    #to_id_arms
                }
            }

            #[must_use]
            pub const fn name(&self) -> &'static str {
                match self {
                    #to_name_arms
                }
            }

            #[must_use]
            pub const fn map_color_id(&self) -> u8 {
                match self {
                    #map_color_id_arms
                }
            }

            #[must_use]
            pub const fn terracotta_color_id(&self) -> u8 {
                match self {
                    #terracotta_color_id_arms
                }
            }

            #[must_use]
            pub const fn texture_diffuse_color(&self) -> u32 {
                match self {
                    #texture_diffuse_color_arms
                }
            }

            #[must_use]
            pub const fn firework_color(&self) -> u32 {
                match self {
                    #firework_color_arms
                }
            }

            #[must_use]
            pub const fn text_color(&self) -> u32 {
                match self {
                    #text_color_arms
                }
            }

            #[must_use]
            pub const fn by_id(id: u8) -> Option<Self> {
                match id {
                    #id_match_arms
                    _ => None,
                }
            }

            #[must_use]
            pub fn by_name(name: &str) -> Option<Self> {
                match name {
                    #name_match_arms
                    _ => None,
                }
            }
        }

        impl From<DyeColor> for String {
            fn from(value: DyeColor) -> Self {
                value.name().to_string()
            }
        }

        impl From<&str> for DyeColor {
            fn from(s: &str) -> Self {
                DyeColor::by_name(s).unwrap_or_default()
            }
        }

        impl From<i8> for DyeColor {
            fn from(s: i8) -> Self {
                if s >= 0 {
                    DyeColor::by_id(s as u8).unwrap_or_default()
                } else {
                    DyeColor::default()
                }
            }
        }

        impl From<u8> for DyeColor {
            fn from(s: u8) -> Self {
                DyeColor::by_id(s).unwrap_or_default()
            }
        }
    }
}
