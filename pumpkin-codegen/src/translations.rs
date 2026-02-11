use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::{collections::BTreeMap, fs};

pub fn build() -> TokenStream {
    let en_us: BTreeMap<String, String> = serde_json::from_str(
        &fs::read_to_string("../assets/en_us.json").expect("en_us is missing"),
    )
    .unwrap();

    let mut constants = TokenStream::new();

    for (name, value) in &en_us {
        let clean_name = name.to_uppercase().replace(['.', '-'], "_");
        let ident = format_ident!("{}", clean_name);

        if value.is_empty() {
            constants.extend(quote! {
                pub const #ident: &str = #name;
            });
        } else {
            constants.extend(quote! {
                #[doc = #value]
                pub const #ident: &str = #name;
            });
        }
    }

    quote! {
        #constants
    }
}
