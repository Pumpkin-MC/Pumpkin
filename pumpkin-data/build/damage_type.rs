use heck::{ToPascalCase, ToShoutySnakeCase};
use proc_macro2::TokenStream;
use quote::quote;
use serde::Deserialize;
use std::collections::HashMap;

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
        let const_ident = crate::ident(name.to_shouty_snake_case());
        let enum_ident = crate::ident(name.to_pascal_case());

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

    let type_name_pairs = enum_variants.iter().map(|variant| {
        let name = variant.to_string();
        let name_lowercase = name.to_lowercase();
        let resource_name = format!("minecraft:{}", name_lowercase);
        quote! {
            #resource_name => Some(Self::#variant)
        }
    });

    let type_to_name_pairs = enum_variants.iter().map(|variant| {
        let name = variant.to_string();
        let name_lowercase = name.to_lowercase();
        let resource_name = format!("minecraft:{}", name_lowercase);
        quote! {
            Self::#variant => #resource_name
        }
    });

    // Create array of all variants for values() method
    let variant_array = enum_variants.iter().map(|variant| {
        quote! {
            DamageType::#variant
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

        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

            #[doc = r" Get all possible damage types"]
            pub fn values() -> &'static [DamageType] {
                static VALUES: &[DamageType] = &[
                    #(#variant_array,)*
                ];
                VALUES
            }

            #[doc = r" Try to parse a damage type from a resource location string"]
            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    #(#type_name_pairs,)*
                    _ => None
                }
            }

            #[doc = r" Get the resource location string for this damage type"]
            pub const fn to_name(&self) -> &'static str {
                match self {
                    #(#type_to_name_pairs,)*
                }
            }
        }
    }
}
