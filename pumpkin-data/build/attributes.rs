use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Deserialize)]
struct Attributes {
    id: u8,
    default_value: f64,
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/attributes.json");

    let attributes: HashMap<String, Attributes> =
        serde_json::from_str(&fs::read_to_string("../assets/attributes.json").unwrap())
            .expect("Failed to parse attributes.json");

    let mut consts = TokenStream::new();

    let mut data_component_vec = attributes.iter().collect::<Vec<_>>();
    data_component_vec.sort_by_key(|(_, i)| i.id);

    for (raw_name, raw_value) in &data_component_vec {
        let pascal_case = format_ident!("{}", raw_name.to_uppercase());

        let id = raw_value.id;
        let default_value = raw_value.default_value;
        consts.extend(quote! {
            pub const #pascal_case: Self = Self {
                id: #id,
                default_value: #default_value,
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
        impl Hash for Attributes {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.id.hash(state);
            }
        }
        impl Attributes {
            #consts
        }
    }
}
