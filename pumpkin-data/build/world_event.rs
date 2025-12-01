use std::{collections::BTreeMap, fs};

use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/world_event.json");

    let events: BTreeMap<String, u16> =
        serde_json::from_str(&fs::read_to_string("../assets/world_event.json").unwrap())
            .expect("Failed to parse world_event.json");
    let variants: Vec<TokenStream> = events
        .into_iter()
        .map(|(event_name, id)| {
            let name = format_ident!("{}", event_name.to_pascal_case());

            quote! {
                #name = #id,
            }
        })
        .collect();

    quote! {
        #[repr(u16)]
        pub enum WorldEvent {
            #(#variants)*
        }
    }
}
