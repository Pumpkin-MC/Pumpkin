use std::collections::HashMap;

use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/entity_statuses.json");

    let events: HashMap<String, u8> =
        serde_json::from_str(include_str!("../../assets/entity_statuses.json"))
            .expect("Failed to parse entity_statuses.json");
    let mut variants = TokenStream::new();

    for (event, id) in events.iter() {
        let name = format_ident!("{}", event.to_pascal_case());
        variants.extend([quote! {
            #name = #id,
        }]);
    }
    quote! {
        #[repr(u8)]
        pub enum EntityStatus {
            #variants
        }
    }
}
