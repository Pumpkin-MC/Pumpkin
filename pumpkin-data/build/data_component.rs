use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Deserialize)]
struct DataComponent {
    id: u8,
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/data_component.json");

    let data_component: HashMap<String, DataComponent> =
        serde_json::from_str(&fs::read_to_string("../assets/data_component.json").unwrap())
            .expect("Failed to parse data_component.json");

    let mut enum_variants = TokenStream::new();
    let mut enum_to_name = TokenStream::new();
    let mut enum_to_id = TokenStream::new();

    let mut data_component_vec = data_component.iter().collect::<Vec<_>>();
    data_component_vec.sort_by_key(|(_, i)| i.id);

    for (raw_name, raw_value) in &data_component_vec {
        let strip_name = raw_name
            .strip_prefix("minecraft:")
            .unwrap()
            .replace("/", "_");
        let pascal_case = format_ident!("{}", strip_name.to_pascal_case());
        let pascal_case_data = format_ident!("{}Impl", strip_name.to_pascal_case());

        // Enum variant
        enum_variants.extend(quote! {
            #pascal_case(#pascal_case_data),
        });

        // Enum -> &str
        enum_to_name.extend(quote! {
            Self::#pascal_case(_) => #raw_name,
        });

        let id = raw_value.id;
        enum_to_id.extend(quote! {
            Self::#pascal_case(_) => #id,
        });
    }

    quote! {
        use crate::data_component_impl::*;

        #[derive(Clone, Debug)]
        pub enum DataComponent {
            #enum_variants
        }

        impl DataComponent {
            pub const fn to_id(&self) -> u8 {
                match self {
                    #enum_to_id
                }
            }
            pub const fn to_name(&self) -> &'static str {
                match self {
                    #enum_to_name
                }
            }
        }
    }
}
