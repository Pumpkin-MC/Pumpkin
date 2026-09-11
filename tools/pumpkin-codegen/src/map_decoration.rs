use std::fs;

use heck::{ToPascalCase, ToShoutySnakeCase};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

#[derive(Deserialize)]
struct MapDecorationEntry {
    name: String,
    asset_name: String,
    show_on_item_frame: bool,
    map_color: i32,
    exploration_map_element: bool,
    track_count: bool,
}

pub fn build() -> TokenStream {
    let decorations: Vec<MapDecorationEntry> =
        serde_json::from_str(&fs::read_to_string("../../assets/map_decorations.json").unwrap())
            .expect("Failed to parse map_decorations.json");

    let mut constants = TokenStream::new();
    let mut all_elements = TokenStream::new();
    let mut name_match_arms = TokenStream::new();

    for (index, entry) in decorations.iter().enumerate() {
        let id = index as u32;
        let name = &entry.name;
        let const_name = format_ident!("{}", name.to_shouty_snake_case());
        let asset_name = &entry.asset_name;
        let show_on_item_frame = entry.show_on_item_frame;
        let map_color = entry.map_color;
        let exploration_map_element = entry.exploration_map_element;
        let track_count = entry.track_count;

        constants.extend(quote! {
            pub const #const_name: MapDecorationType = MapDecorationType {
                id: #id,
                name: #name,
                asset_name: #asset_name,
                show_on_item_frame: #show_on_item_frame,
                map_color: #map_color,
                exploration_map_element: #exploration_map_element,
                track_count: #track_count,
            };
        });

        all_elements.extend(quote! {
            MapDecorationType::#const_name,
        });

        name_match_arms.extend(quote! {
            #name => Some(&Self::#const_name),
        });
    }

    quote! {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct MapDecorationType {
            pub id: u32,
            pub name: &'static str,
            pub asset_name: &'static str,
            pub show_on_item_frame: bool,
            pub map_color: i32,
            pub exploration_map_element: bool,
            pub track_count: bool,
        }

        impl MapDecorationType {
            #constants

            pub const ALL: &'static [MapDecorationType] = &[
                #all_elements
            ];

            #[must_use]
            pub const fn from_id(id: u32) -> Option<&'static MapDecorationType> {
                if (id as usize) < Self::ALL.len() {
                    Some(&Self::ALL[id as usize])
                } else {
                    None
                }
            }

            #[must_use]
            pub fn from_name(name: &str) -> Option<&'static MapDecorationType> {
                match name {
                    #name_match_arms
                    _ => None,
                }
            }
        }
    }
}
