use std::fs;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::array_to_tokenstream;

/// Generates the `TokenStream` for the `GameEvent` enum.
pub fn build() -> TokenStream {
    let game_events: Vec<String> =
        serde_json::from_str(&fs::read_to_string("../../assets/game_event.json").unwrap())
            .expect("Failed to parse game_event.json");
    let variants = array_to_tokenstream(&game_events);
    let names = game_events.iter().map(|name| {
        let variant = format_ident!("{}", heck::ToPascalCase::to_pascal_case(name.as_str()));
        quote! { Self::#variant => #name, }
    });
    let from_names = game_events.iter().map(|name| {
        let variant = format_ident!("{}", heck::ToPascalCase::to_pascal_case(name.as_str()));
        quote! { #name => Some(Self::#variant), }
    });

    quote! {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum GameEvent {
            #variants
        }

        impl GameEvent {
            #[must_use]
            pub const fn name(self) -> &'static str {
                match self {
                    #(#names)*
                }
            }

            #[must_use]
            pub fn from_name(name: &str) -> Option<Self> {
                match name.strip_prefix("minecraft:").unwrap_or(name) {
                    #(#from_names)*
                    _ => None,
                }
            }
        }
    }
}
