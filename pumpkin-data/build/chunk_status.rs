use std::fs;

use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/chunk_status.json");

    let chunk_status: Vec<String> =
        serde_json::from_str(&fs::read_to_string("../assets/chunk_status.json").unwrap())
            .expect("Failed to parse chunk_status.json");
    let mut variants = TokenStream::new();

    for status in chunk_status.iter() {
        let full_name = format!("minecraft:{status}");
        let name = format_ident!("{}", status.to_pascal_case());
        variants.extend([quote! {
            #[serde(rename = #full_name)]
            #name,
        }]);
    }
    quote! {
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
        pub enum ChunkStatus {
            #variants
        }
    }
}
