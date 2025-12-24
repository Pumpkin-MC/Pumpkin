use std::{collections::BTreeMap, fs};

use heck::ToPascalCase;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::LitInt;

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/entity_statuses.json");

    let events: BTreeMap<String, u8> =
        serde_json::from_str(&fs::read_to_string("../assets/entity_statuses.json").unwrap())
            .expect("Failed to parse entity_statuses.json");
    let variants: Vec<TokenStream> = events
        .into_iter()
        .map(|(event_name, id)| {
            let name = format_ident!("{}", event_name.to_pascal_case());
            let id_lit = LitInt::new(&id.to_string(), Span::call_site());

            quote! {
                #name = #id_lit
            }
        })
        .collect();
    quote! {
        #[repr(u8)]
        pub enum EntityStatus {
            #(#variants),*
        }
    }
}
