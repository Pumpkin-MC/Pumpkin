use std::fs;

use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::array_to_tokenstream;

pub fn build() -> TokenStream {
    let mut sound: Vec<String> =
        serde_json::from_str(&fs::read_to_string("../assets/sounds.json").unwrap())
            .expect("Failed to parse sounds.json");

    sound.sort();

    let variants = array_to_tokenstream(&sound);

    let lookup_table = sound.iter().map(|s| {
        let variant_name = format_ident!("{}", s.to_pascal_case());
        quote! { (#s, Self::#variant_name) }
    });

    let variants_list = sound.iter().map(|s| {
        let variant_name = format_ident!("{}", s.to_pascal_case());
        quote! { Self::#variant_name, }
    });

    let names_list = sound.iter().map(|s| quote! { #s });

    quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[repr(u16)]
        pub enum Sound {
            #variants
        }

        impl Sound {
            const NAMES: &[&str] = &[ #(#names_list),* ];

            const LOOKUP: &[(&str, Sound)] = &[
                    #(#lookup_table),*
            ];

            pub fn from_name(name: &str) -> Option<Self> {
                Self::LOOKUP
                    .binary_search_by_key(&name, |&(k, _)| k)
                    .ok()
                    .map(|idx| Self::LOOKUP[idx].1)
            }

           pub const fn to_name(&self) -> &'static str {
                Self::NAMES[*self as usize]
            }

            pub fn slice() -> &'static [Self] {
                &[#(#variants_list)*]
            }
        }
    }
}
