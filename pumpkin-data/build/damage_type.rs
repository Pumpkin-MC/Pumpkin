use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;
use std::collections::HashMap;
use syn::{Ident, LitInt};

#[derive(Deserialize)]
struct DamageTypeEntry {
    id: u32,
    components: DamageTypeData,
}

#[derive(Deserialize)]
pub struct DamageTypeData {
    death_message_type: Option<DeathMessageType>,
    exhaustion: f32,
    message_id: String,
    scaling: DamageScaling,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum DamageScaling {
    Never,
    WhenCausedByLivingNonPlayer,
    Always,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum DeathMessageType {
    Default,
    FallVariants,
    IntentionalGameDesign,
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/damage_type.json");

    let damage_types: HashMap<String, DamageTypeEntry> =
        serde_json::from_str(include_str!("../../assets/damage_type.json"))
            .expect("Failed to parse damage_type.json");

    let mut constants = Vec::new();
    let mut enum_variants = Vec::new();

    for (name, entry) in damage_types {
        let const_ident = format_ident!("{}", name.to_shouty_snake_case());

        enum_variants.push(const_ident.clone());

        let data = &entry.components;
        let death_message_type = match &data.death_message_type {
            Some(msg) => {
                let msg_ident = Ident::new(&format!("{:?}", msg), proc_macro2::Span::call_site());
                quote! { Some(DeathMessageType::#msg_ident) }
            }
            None => quote! { None },
        };

        let exhaustion = data.exhaustion;
        let message_id = &data.message_id;
        let scaling_ident = Ident::new(
            &format!("{:?}", data.scaling),
            proc_macro2::Span::call_site(),
        );
        let scaling = quote! {DamageScaling::#scaling_ident};
        let id_lit = LitInt::new(&entry.id.to_string(), proc_macro2::Span::call_site());

        constants.push(quote! {
            pub const #const_ident: DamageType = DamageType {
                death_message_type: #death_message_type,
                exhaustion: #exhaustion,
                message_id: #message_id,
                scaling: #scaling,
                id: #id_lit,
            };
        });
    }

    let type_name_pairs = enum_variants.iter().map(|variant| {
        let name = variant.to_string();
        let name_lowercase = name.to_lowercase();
        let resource_name = format!("minecraft:{}", name_lowercase);
        quote! {
            #resource_name => Some(Self::#variant)
        }
    });

    quote! {
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct DamageType {
            pub death_message_type: Option<DeathMessageType>,
            pub exhaustion: f32,
            pub message_id: &'static str,
            pub scaling: DamageScaling,
            pub id: u32,
        }

        #[derive(Clone, Copy, Debug, PartialEq)]
        pub enum DeathMessageType {
            Default,
            FallVariants,
            IntentionalGameDesign,
        }

        #[derive(Clone, Copy, Debug, PartialEq)]
        pub enum DamageScaling {
            Never,
            WhenCausedByLivingNonPlayer,
            Always,
        }

        impl DamageType {
            #(#constants)*

            #[doc = r" Try to parse a damage type from a resource location string"]
            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    #(#type_name_pairs,)*
                    _ => None
                }
            }

        }
    }
}
