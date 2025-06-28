use std::{collections::HashMap, fs};

use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/composter_increase_chance.json");

    let composter_increase_chance: HashMap<u16, f32> = serde_json::from_str(
        &fs::read_to_string("../assets/composter_increase_chance.json").unwrap(),
    )
    .expect("Failed to parse composter_increase_chance.json");
    let mut variants = TokenStream::new();

    for (item_id, chance) in composter_increase_chance {
        variants.extend(quote! {
            #item_id => Some(#chance),
        });
    }
    quote! {
        #[must_use]
        pub const fn get_composter_increase_chance_from_item_id(item_id: u16) -> Option<f32> {
            match item_id {
                #variants
                _ => None,
            }
        }
    }
}
