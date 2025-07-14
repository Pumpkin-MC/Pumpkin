use heck::ToPascalCase;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use std::collections::HashMap;
use std::fs;
use syn::Lifetime;

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/data_component.json");
    println!("cargo:rerun-if-changed=../assets/data_component_lifetime.json");

    let data_component: HashMap<String, u8> =
        serde_json::from_str(&fs::read_to_string("../assets/data_component.json").unwrap())
            .expect("Failed to parse data_component.json");
    let lifetime: HashMap<String, String> = serde_json::from_str(
        &fs::read_to_string("../assets/data_component_lifetime.json").unwrap(),
    )
    .expect("Failed to parse data_component_lifetime.json");

    let mut enum_variants = TokenStream::new();
    let mut enum_to_name = TokenStream::new();
    let mut enum_to_id = TokenStream::new();

    let mut data_component_vec = data_component.iter().collect::<Vec<_>>();
    data_component_vec.sort_by_key(|(_, i)| **i);

    for (raw_name, raw_value) in &data_component_vec {
        let strip_name = raw_name
            .strip_prefix("minecraft:")
            .unwrap()
            .replace("/", "_");
        let pascal_case = format_ident!("{}", strip_name.to_pascal_case());
        let pascal_case_data = format_ident!("{}Impl", strip_name.to_pascal_case());

        // Enum variant
        if let Some(str) = lifetime.get(*raw_name) {
            let lifetime = Lifetime::new(str, Span::call_site());
            enum_variants.extend(quote! {
                #pascal_case(#pascal_case_data<#lifetime>),
            });
        } else {
            enum_variants.extend(quote! {
                #pascal_case(#pascal_case_data),
            });
        }

        // Enum -> &str
        enum_to_name.extend(quote! {
            Self::#pascal_case(_) => #raw_name,
        });

        enum_to_id.extend(quote! {
            Self::#pascal_case(_) => #raw_value,
        });
    }

    quote! {
        use crate::data_component_impl::*;

        #[derive(Clone, Debug)]
        pub enum DataComponent<'a> {
            #enum_variants
        }

        impl DataComponent<'_> {
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
