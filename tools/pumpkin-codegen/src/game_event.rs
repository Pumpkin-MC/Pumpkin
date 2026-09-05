use std::fs;

use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// Generates the `TokenStream` for the `GameEvent` enum.
pub fn build() -> TokenStream {
    let game_events: Vec<String> =
        serde_json::from_str(&fs::read_to_string("../../assets/game_event.json").unwrap())
            .expect("Failed to parse game_event.json");

    let mut variants = TokenStream::new();
    let mut from_name_match = TokenStream::new();
    let mut to_name_match = TokenStream::new();

    for event in &game_events {
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
    }

    quote! {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum GameEvent {
            #variants
        }

        impl GameEvent {
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
