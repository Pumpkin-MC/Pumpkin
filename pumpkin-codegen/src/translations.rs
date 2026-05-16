use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::{collections::BTreeMap, fs};

pub fn build() -> TokenStream {
    let java_json: BTreeMap<String, String> = serde_json::from_str(
        &fs::read_to_string("../assets/en_us_java.json").expect("en_us_java is missing"),
    )
    .unwrap();

    let mut java_constants = TokenStream::new();
    for (name, value) in &java_json {
        let ident = to_valid_ident(name);
        let doc = if !value.is_empty() {
            quote!(#[doc = #value])
        } else {
            quote!()
        };
        java_constants.extend(quote! {
            #doc
            pub const #ident: &str = #name;
        });
    }

    let bedrock_content =
        fs::read_to_string("../assets/en_us_bedrock.lang").expect("en_us_bedrock is missing");
    let mut bedrock_constants = TokenStream::new();

    for line in bedrock_content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('/') {
            continue;
        }

        if let Some((name, value)) = line.split_once('=') {
            let name = name.trim();
            let value = value.trim();
            let ident = to_valid_ident(name);

            let doc = if !value.is_empty() {
                quote!(#[doc = #value])
            } else {
                quote!()
            };
            bedrock_constants.extend(quote! {
                #doc
                pub const #ident: &str = #name;
            });
        }
    }

    // --- Final Assembly ---
    quote! {
        pub mod java {
            #java_constants
        }
        pub mod bedrock {
            #bedrock_constants
        }
    }
}

fn to_valid_ident(name: &str) -> Ident {
    let mut clean = name.to_uppercase().replace(['.', ':', '-'], "_");

    if clean.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        clean.insert(0, '_');
    }

    format_ident!("{}", clean)
}
