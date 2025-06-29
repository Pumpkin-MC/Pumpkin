use proc_macro2::TokenStream;
use quote::quote;
use std::{collections::HashMap, fs};
pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/flower_pot_transformations.json");

    let flower_pot_transformation: HashMap<u16, u16> = serde_json::from_str(
        &fs::read_to_string("../assets/flower_pot_transformations.json").unwrap(),
    )
    .expect("Failed to parse flower_pot_transformations.json");
    let mut variants = TokenStream::new();

    for (item_id, potted_block_id) in flower_pot_transformation {
        variants.extend(quote! {
            #item_id => Some(#potted_block_id),
        });
    }
    quote! {
        #[must_use]
        pub const fn get_potted_item(item_id: u16) -> Option<u16> {
            match item_id {
                #variants
                _ => None,
            }
        }
    }
}
