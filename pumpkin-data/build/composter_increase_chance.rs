use std::{collections::BTreeMap, fs};

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::LitInt;

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/composter_increase_chance.json");

    let composter_increase_chance: BTreeMap<u16, f32> = serde_json::from_str(
        &fs::read_to_string("../assets/composter_increase_chance.json").unwrap(),
    )
    .expect("Failed to parse composter_increase_chance.json");
    let match_arms: Vec<TokenStream> = composter_increase_chance
        .into_iter() // Consume the map for efficiency
        .map(|(item_id, chance)| {
            let item_id_lit = LitInt::new(&item_id.to_string(), Span::call_site());
            let chance_lit = chance;

            quote! {
                #item_id_lit => Some(#chance_lit),
            }
        })
        .collect();
    quote! {
        #[must_use]
        pub const fn get_composter_increase_chance_from_item_id(item_id: u16) -> Option<f32> {
            match item_id {
                #(#match_arms)*
                _ => None,
            }
        }
    }
}
