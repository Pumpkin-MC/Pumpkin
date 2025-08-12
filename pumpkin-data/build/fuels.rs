use proc_macro2::TokenStream;
use quote::quote;
use std::{collections::BTreeMap, fs};
pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/fuels.json");

    let fuels: BTreeMap<u16, u16> =
        serde_json::from_str(&fs::read_to_string("../assets/fuels.json").unwrap())
            .expect("Failed to parse fuels.json");

    let fuel_list_tokens = fuels
        .iter()
        .map(|(item_id, burn_ticks)| {
            quote! {
                (#item_id, #burn_ticks)
            }
        })
        .collect::<Vec<_>>();
    let fuel_list_len = fuel_list_tokens.len();

    let burn_tick_list_tokens = fuels
        .iter()
        .map(|(item_id, burn_ticks)| {
            quote! {
                #item_id => Some(#burn_ticks),
            }
        })
        .collect::<Vec<_>>();

    quote! {
        pub static FUELS: [(u16,u16); #fuel_list_len] = [
                #(#fuel_list_tokens),*
        ];

        #[must_use]
        pub const fn get_item_burn_ticks(item_id: u16) -> Option<u16> {
            match item_id {
                #(#burn_tick_list_tokens)*
                _ => None,
            }
        }

        #[must_use]
        pub const fn is_fuel(item_id: u16) -> bool {
            get_item_burn_ticks(item_id).is_some()
        }
    }
}
