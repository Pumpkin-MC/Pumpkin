use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;
use std::{collections::BTreeMap, fs};
use syn::{Ident, LitInt};

/// Raw deserialization wrapper for a single damage type entry from `damage_type.json`.
#[derive(Deserialize)]
struct DamageTypeEntry {
    /// Numeric registry ID for this damage type.
    id: u8,
    /// Component data describing the behavior and messaging for this damage type.
    components: DamageTypeData,
}

/// Component data describing the behavior and messaging for a damage type.
#[derive(Deserialize)]
pub struct DamageTypeData {
    /// Which death-message variant to display; defaults to `Default` when absent.
    death_message_type: Option<DeathMessageType>,
    /// Exhaustion (hunger) cost applied to the player when taking this damage.
    exhaustion: f32,
    /// Sound and visual effect triggered when this damage is received.
    effects: Option<DamageEffects>,
    /// Translation key fragment used to look up the death message string.
    message_id: String,
    /// Whether and when this damage type scales with game difficulty.
    scaling: DamageScaling,
}

/// Sound and visual effect applied when an entity takes this kind of damage.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum DamageEffects {
    Hurt,
    Thorns,
    Drowning,
    Burning,
    Poking,
    Freezing,
}

/// Determines whether this damage type scales with difficulty.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum DamageScaling {
    Never,
    WhenCausedByLivingNonPlayer,
    Always,
}

/// Controls which death message variant is displayed for this damage type.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum DeathMessageType {
    Default,
    FallVariants,
    IntentionalGameDesign,
}

/// Generates the `TokenStream` for the `DamageType` struct, its associated enums, and constants.
pub fn build() -> TokenStream {
    let damage_types: BTreeMap<String, DamageTypeEntry> =
        serde_json::from_str(&fs::read_to_string("../assets/damage_type.json").unwrap())
            .expect("Failed to parse damage_type.json");

    let mut constants = Vec::new();
    let mut type_from_name = TokenStream::new();

    for (name, entry) in damage_types {
        let const_ident = format_ident!("{}", name.to_shouty_snake_case());
        let resource_name = name.to_lowercase();

        type_from_name.extend(quote! {
            #resource_name => Some(Self::#const_ident),
        });

        let data = &entry.components;
        let death_message_type = if let Some(msg) = &data.death_message_type {
            let msg_ident = Ident::new(&format!("{msg:?}"), proc_macro2::Span::call_site());
            quote! { DeathMessageType::#msg_ident }
        } else {
            quote! { DeathMessageType::Default }
        };

        let effects = if let Some(msg) = &data.effects {
            let msg_ident = Ident::new(&format!("{msg:?}"), proc_macro2::Span::call_site());
            quote! { Some(DamageEffects::#msg_ident) }
        } else {
            quote! { None }
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
                effects: #effects,
                message_id: #message_id,
                scaling: #scaling,
                id: #id_lit,
            };
        });
    }

    quote! {
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct DamageType {
            pub death_message_type: DeathMessageType,
            pub exhaustion: f32,
            pub effects: Option<DamageEffects>,
            pub message_id: &'static str,
            pub scaling: DamageScaling,
            pub id: u8,
        }

        #[derive(Clone, Copy, Debug, PartialEq)]
        pub enum DeathMessageType {
            Default,
            FallVariants,
            IntentionalGameDesign,
        }

        #[derive(Clone, Copy, Debug, PartialEq)]
        pub enum DamageEffects {
            Hurt,
            Thorns,
            Drowning,
            Burning,
            Poking,
            Freezing,
        }

        #[derive(Clone, Copy, Debug, PartialEq)]
        pub enum DamageScaling {
            Never,
            WhenCausedByLivingNonPlayer,
            Always,
        }

        impl DamageType {
            #(#constants)*

            #[doc = r" Try to parse a damage type from a resource location string."]
            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    #type_from_name
                    _ => None
                }
            }

        }
    }
}
