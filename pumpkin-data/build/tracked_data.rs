use std::fs;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/tracked_data.json");

    let mut handlers: Vec<(String, u8)> =
        serde_json::from_str::<std::collections::HashMap<String, u8>>(
            &fs::read_to_string("../assets/tracked_data.json").unwrap(),
        )
        .expect("Failed to parse tracked_data.json")
        .into_iter()
        .collect();

    handlers.sort_by_key(|&(_, id)| id);

    let constants = handlers.iter().map(|(name, id)| {
        let ident = format_ident!("DATA_{}", name.to_uppercase());
        quote! {
            pub const #ident: u8 = #id;
        }
    });

    quote! {
        pub struct TrackedData;

        impl TrackedData {
            #(#constants)*
        }
    }
}
