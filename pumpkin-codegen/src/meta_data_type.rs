use std::{collections::BTreeMap, fs};

use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn build() -> TokenStream {
    let handlers_map: BTreeMap<String, i32> =
        serde_json::from_str(&fs::read_to_string("../assets/meta_data_type.json").unwrap())
            .expect("Failed to parse meta_data_type.json");

    let mut handlers: Vec<(&String, &i32)> = handlers_map.iter().collect();
    handlers.sort_by_key(|&(_, id)| id);

    let variants = handlers.iter().map(|(name, _)| {
        let ident = format_ident!("{}", name.to_pascal_case());
        quote! { #ident }
    });

    let from_id = handlers.iter().map(|(name, id)| {
        let ident = format_ident!("{}", name.to_pascal_case());
        quote! { #id => Some(Self::#ident), }
    });

    let to_id = handlers.iter().map(|(name, id)| {
        let ident = format_ident!("{}", name.to_pascal_case());
        quote! { Self::#ident => #id, }
    });

    quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[repr(i32)]
        pub enum MetaDataType {
            #(#variants),*
        }

        impl MetaDataType {
            pub fn from_id(id: i32) -> Option<Self> {
                match id {
                    #(#from_id)*
                    _ => None,
                }
            }

            pub const fn id(&self) -> i32 {
                match self {
                    #(#to_id)*
                }
            }
        }
    }
}
