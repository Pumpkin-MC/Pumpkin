use std::{collections::BTreeMap, fs};

use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::entity_type::EntityType;

/// Generates a narrow entity-type -> hurt sound lookup from the `hurt_sound` field in
/// `entities.json`.
pub fn build() -> TokenStream {
    let entities: BTreeMap<String, EntityType> =
        serde_json::from_str(&fs::read_to_string("../assets/entities.json").unwrap())
            .expect("Failed to parse entities.json");

    let lookup_arms = entities
        .into_iter()
        .filter_map(|(entity_name, entity)| entity.hurt_sound.map(|sound_name| (entity_name, sound_name)))
        .map(|(entity_name, sound_name)| {
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
