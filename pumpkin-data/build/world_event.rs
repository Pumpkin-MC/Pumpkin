use std::collections::HashMap;

use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/world_event.json");

    let events: HashMap<String, u16> =
        serde_json::from_str(include_str!("../../assets/world_event.json"))
            .expect("Failed to parse world_event.json");
    let mut variants = TokenStream::new();

    for (event, id) in events.iter() {
        let name = format_ident!("{}", event.to_pascal_case());
        variants.extend([quote! {
            #name = #id,
        }]);
    }
    quote! {
        #[repr(u16)]
        pub enum WorldEvent {
            #variants
        }
    }
}
