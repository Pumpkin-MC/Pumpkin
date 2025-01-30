use proc_macro2::TokenStream;
use quote::quote;
use serde::Deserialize;
use std::collections::HashMap;
use heck::{ToShoutySnakeCase, ToPascalCase};

#[derive(Deserialize)]
struct DamageTypeEntry {
    id: u32,
    components: DamageTypeData,
}

#[derive(Deserialize)]
pub struct DamageTypeData {
    death_message_type: Option<String>,
    exhaustion: f32,
    message_id: String,
    scaling: String,
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/damage_type.json");

    let damage_types: HashMap<String, DamageTypeEntry> = 
        serde_json::from_str(include_str!("../../assets/damage_type.json"))
            .expect("Failed to parse damage_type.json");

    let mut constants = Vec::new();
    let mut enum_variants = Vec::new();

    for (name, entry) in damage_types {
        let const_ident = crate::ident(&name.to_shouty_snake_case());
        let enum_ident = crate::ident(&name.to_pascal_case());
        
        enum_variants.push(enum_ident.clone());
        
        let data = &entry.components;
        let death_message_type = match &data.death_message_type {
            Some(msg) => quote! { Some(#msg) },
            None => quote! { None },
        };
        
        let exhaustion = data.exhaustion;
        let message_id = &data.message_id;
        let scaling = &data.scaling;
        let id = entry.id;
        
        constants.push(quote! {
            pub const #const_ident: DamageTypeData = DamageTypeData {
                death_message_type: #death_message_type,
                exhaustion: #exhaustion,
                message_id: #message_id,
                scaling: #scaling,
                id: #id,
            };
        });
    }

    let enum_arms = enum_variants.iter().map(|variant| {
        let const_name = variant.to_string().to_shouty_snake_case();
        let const_ident = crate::ident(&const_name);
        quote! {
            DamageType::#variant => &#const_ident,
        }
    });

    quote! {
        #[derive(Clone, Debug)]
        pub struct DamageTypeData {
            pub death_message_type: Option<&'static str>,
            pub exhaustion: f32,
            pub message_id: &'static str,
            pub scaling: &'static str,
            pub id: u32,
        }

        #(#constants)*

        #[derive(Clone, Copy, Debug)]
        #[repr(u8)]
        pub enum DamageType {
            #(#enum_variants,)*
        }

        impl DamageType {
            pub const fn data(&self) -> &'static DamageTypeData {
                match self {
                    #(#enum_arms)*
                }
            }
        }
    }
}
