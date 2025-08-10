use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashMap;
use std::fs;

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/data_component.json");

    let data_component: HashMap<String, u8> =
        serde_json::from_str(&fs::read_to_string("../assets/data_component.json").unwrap())
            .expect("Failed to parse data_component.json");

    let mut enum_variants = TokenStream::new();
    let mut id_to_enum = TokenStream::new();
    let mut enum_to_name = TokenStream::new();
    let mut data_component_vec = data_component.iter().collect::<Vec<_>>();
    data_component_vec.sort_by_key(|(_, i)| **i);

    for (raw_name, raw_value) in &data_component_vec {
        let strip_name = raw_name
            .strip_prefix("minecraft:")
            .unwrap()
            .replace("/", "_");
        let pascal_case = format_ident!("{}", strip_name.to_pascal_case());

        // Enum variant

        enum_variants.extend(quote! {
            #pascal_case = #raw_value,
        });

        id_to_enum.extend(quote! {
            #raw_value => Some(DataComponent::#pascal_case),
        });

        // Enum -> &str
        enum_to_name.extend(quote! {
            Self::#pascal_case => #raw_name,
        });
    }

    quote! {
        use crate::data_component_impl::*;

        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
        #[repr(u8)]
        pub enum DataComponent {
            #enum_variants
        }

        impl DataComponent {
            pub const fn to_id(self) -> u8 {
                self as u8
            }
            pub const fn try_from_id(id: u8) -> Option<DataComponent> {
                match id {
                    #id_to_enum
                    _ => None,
                }
            }
            pub const fn to_name(self) -> &'static str {
                match self {
                    #enum_to_name
                }
            }
        }
    }
}
