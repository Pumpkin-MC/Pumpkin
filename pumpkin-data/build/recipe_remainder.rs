use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::{collections::BTreeMap, fs};
use syn::LitInt;
pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/recipe_remainder.json");

    let remainder: BTreeMap<u16, u16> =
        serde_json::from_str(&fs::read_to_string("../assets/recipe_remainder.json").unwrap())
            .expect("Failed to parse recipe_remainder.json");
    let match_arms: Vec<TokenStream> = remainder
        .into_iter()
        .map(|(item_id, remainder_id)| {
            let item_id_lit = LitInt::new(&item_id.to_string(), Span::call_site());
            let remainder_id_lit = LitInt::new(&remainder_id.to_string(), Span::call_site());

            quote! {
                #item_id_lit => Some(#remainder_id_lit),
            }
        })
        .collect();
    quote! {
        #[must_use]
        pub const fn get_recipe_remainder_id(item_id: u16) -> Option<u16> {
            match item_id {
                #(#match_arms)*
                _ => None,
            }
        }
    }
}
