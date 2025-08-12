use proc_macro2::TokenStream;
use quote::quote;
use std::{collections::BTreeMap, fs};
pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/recipe_remainder.json");

    let remainder: BTreeMap<u16, u16> =
        serde_json::from_str(&fs::read_to_string("../assets/recipe_remainder.json").unwrap())
            .expect("Failed to parse recipe_remainder.json");
    let mut variants = TokenStream::new();

    for (item_id, remainder_id) in remainder {
        variants.extend(quote! {
            #item_id => Some(#remainder_id),
        });
    }
    quote! {
        #[must_use]
        pub const fn get_recipe_remainder_id(item_id: u16) -> Option<u16> {
            match item_id {
                #variants
                _ => None,
            }
        }
    }
}
