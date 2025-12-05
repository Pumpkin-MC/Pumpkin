use heck::ToShoutySnakeCase;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use syn::LitInt;

#[derive(Deserialize)]
struct Attributes {
    id: u8,
    default_value: f64,
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/attributes.json");

    let attributes: BTreeMap<String, Attributes> =
        serde_json::from_str(&fs::read_to_string("../assets/attributes.json").unwrap())
            .expect("Failed to parse attributes.json");

    let mut sorted_attributes: Vec<(String, Attributes)> = attributes.into_iter().collect();
    sorted_attributes.sort_by_key(|(_, raw)| raw.id);

    let mut constant_defs = Vec::new();

    for (raw_name, raw_value) in sorted_attributes {
        let constant_ident = format_ident!("{}", raw_name.to_shouty_snake_case());

        let id_lit = LitInt::new(&raw_value.id.to_string(), Span::call_site());
        let default_value_lit = raw_value.default_value;

        constant_defs.push(quote! {
            pub const #constant_ident: Self = Self {
                id: #id_lit,
                default_value: #default_value_lit,
            };
        });
    }

    quote! {
        use std::hash::Hash;

        #[derive(Clone, Debug)]
        pub struct Attributes {
            pub id: u8,
            pub default_value: f64,
        }
        impl PartialEq for Attributes {
            fn eq(&self, other: &Self) -> bool {
                self.id == other.id
            }
        }
        impl Eq for Attributes {}
        impl Hash for Attributes {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.id.hash(state);
            }
        }
        impl Attributes {
            #(#constant_defs)*
        }
    }
}
