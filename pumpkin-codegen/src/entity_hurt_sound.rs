use std::{collections::BTreeMap, fs};

use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// Generates a narrow entity-type -> hurt sound lookup from `entity_hurt_sound.json`.
pub fn build() -> TokenStream {
    let sounds: BTreeMap<String, String> =
        serde_json::from_str(&fs::read_to_string("../assets/entity_hurt_sound.json").unwrap())
            .expect("Failed to parse entity_hurt_sound.json");

    let lookup_arms = sounds.into_iter().map(|(entity_name, sound_name)| {
        let entity_ident = format_ident!("{}", entity_name.to_uppercase());
        let sound_ident = format_ident!("{}", sound_name.to_pascal_case());

        quote! {
            id if id == EntityType::#entity_ident.id => Some(Sound::#sound_ident),
        }
    });

    quote! {
        use crate::entity::EntityType;
        use crate::sound::Sound;

        pub const fn hurt_sound_for_entity_type(entity_type: &'static EntityType) -> Option<Sound> {
            match entity_type.id {
                #(#lookup_arms)*
                _ => None,
            }
        }
    }
}
