use std::fs;

use heck::{ToPascalCase, ToShoutySnakeCase};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

#[derive(Deserialize)]
struct MapColorEntry {
    id: u8,
    name: String,
    col: u32,
    hex: String,
    rgb: (u8, u8, u8),
}

pub fn build() -> TokenStream {
    let colors: Vec<MapColorEntry> =
        serde_json::from_str(&fs::read_to_string("../../assets/map_colors.json").unwrap())
            .expect("Failed to parse map_colors.json");

    let mut constants = TokenStream::new();
    let mut all_elements = TokenStream::new();
    let mut id_match_arms = TokenStream::new();
    let mut name_match_arms = TokenStream::new();

    let mut seen_names = std::collections::HashSet::new();

    for entry in &colors {
        let id = entry.id;
        let name = &entry.name;
        let const_name = if name == "none" {
            format_ident!("NONE_{}", id)
        } else {
            format_ident!("{}", name.to_shouty_snake_case())
        };

        let col = entry.col;
        let (r, g, b) = entry.rgb;

        constants.extend(quote! {
            pub const #const_name: MapColor = MapColor {
                id: #id,
                name: #name,
                col: #col,
                rgb: (#r, #g, #b),
            };
        });

        all_elements.extend(quote! {
            MapColor::#const_name,
        });

        id_match_arms.extend(quote! {
            #id => Some(&Self::#const_name),
        });

        if seen_names.insert(name.clone()) {
            name_match_arms.extend(quote! {
                #name => Some(&Self::#const_name),
            });
        }
    }

    quote! {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct MapColor {
            pub id: u8,
            pub name: &'static str,
            pub col: u32,
            pub rgb: (u8, u8, u8),
        }

        impl MapColor {
            #constants

            pub const ALL: &'static [MapColor] = &[
                #all_elements
            ];

            #[must_use]
            pub const fn from_id(id: u8) -> Option<&'static MapColor> {
                if (id as usize) < Self::ALL.len() {
                    Some(&Self::ALL[id as usize])
                } else {
                    None
                }
            }

            #[must_use]
            pub fn from_name(name: &str) -> Option<&'static MapColor> {
                match name {
                    #name_match_arms
                    _ => None,
                }
            }
        }
    }
}
