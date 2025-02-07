use std::collections::HashMap;

use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::quote;
use serde::Deserialize;
use syn::LitInt;

use crate::ident;

#[derive(Deserialize)]
pub struct JSONStruct {
    id: u16,
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/entities.json");

    let json: HashMap<String, JSONStruct> =
        serde_json::from_str(include_str!("../../assets/entities.json"))
            .expect("Failed to parse sound_category.json");
    let mut variants = TokenStream::new();

    for (item, id) in json.iter() {
        let id = id.id as u8;
        let id_lit = LitInt::new(&id.to_string(), proc_macro2::Span::call_site());
        let name = ident(item.to_pascal_case());
        variants.extend([quote! {
            #name = #id_lit,
        }]);
    }

    let type_from_raw_id_arms = json
        .iter()
        .map(|sound| {
            let id = &sound.1.id;
            let name = ident(sound.0.to_pascal_case());

            quote! {
                #id => Some(Self::#name),
            }
        })
        .collect::<TokenStream>();

    let type_from_name = json
        .iter()
        .map(|sound| {
            let id = &sound.0;
            let name = ident(sound.0.to_pascal_case());

            quote! {
                #id => Some(Self::#name),
            }
        })
        .collect::<TokenStream>();

    quote! {
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum EntityType {
            #variants
        }

        impl EntityType {
            pub const fn from_raw(id: u16) -> Option<Self> {
                match id {
                    #type_from_raw_id_arms
                    _ => None
                }
            }

            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    #type_from_name
                    _ => None
                }
            }
        }
    }
}
