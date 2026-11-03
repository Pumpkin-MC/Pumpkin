use std::fs;

use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

#[derive(serde::Deserialize)]
pub(crate) struct GameEvent {
    pub(crate) name: String,
    notification_radius: i32,
}

/// Generates game events and their listener position source types.
pub fn build() -> TokenStream {
    let game_events: Vec<GameEvent> =
        serde_json::from_str(&fs::read_to_string("../../assets/game_event.json").unwrap())
            .expect("Failed to parse game_event.json");
    let position_source_types: Vec<String> = serde_json::from_str(
        &fs::read_to_string("../../assets/position_source_type.json").unwrap(),
    )
    .expect("Failed to parse position_source_type.json");
    let position_source_variants = crate::array_to_tokenstream(&position_source_types);

    let mut variants = TokenStream::new();
    let mut from_name_match = TokenStream::new();
    let mut to_name_match = TokenStream::new();
    let mut radius_match = TokenStream::new();

    for definition in &game_events {
        let event = &definition.name;
        let radius = definition.notification_radius;
        let ident = format_ident!("{}", event.to_pascal_case());
        let namespaced = format!("minecraft:{event}");
        variants.extend(quote! {
            #ident,
        });
        from_name_match.extend(quote! {
            #event | #namespaced => Some(Self::#ident),
        });
        to_name_match.extend(quote! {
            Self::#ident => #event,
        });
        radius_match.extend(quote! { Self::#ident => #radius, });
    }

    quote! {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        #[repr(i32)]
        pub enum PositionSourceType {
            #position_source_variants
        }

        impl PositionSourceType {
            #[must_use]
            pub const fn to_id(self) -> i32 {
                self as i32
            }
        }

        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum GameEvent {
            #variants
        }

        impl GameEvent {
            #[must_use]
            pub const fn notification_radius(self) -> i32 {
                match self { #radius_match }
            }

            #[must_use]
            pub const fn name(&self) -> &'static str {
                match self {
                    #to_name_match
                }
            }

            #[must_use]
            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    #from_name_match
                    _ => None,
                }
            }
        }
    }
}
