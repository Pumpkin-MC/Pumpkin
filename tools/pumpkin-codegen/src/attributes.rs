use heck::ToShoutySnakeCase;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use syn::LitInt;

/// Raw deserialization shape for a single attribute entry from `attributes.json`.
#[derive(Deserialize)]
struct Attributes {
    /// Numeric registry ID for this attribute.
    id: u8,
    /// Default numeric value applied to entities that do not override this attribute.
    default_value: f64,
}

/// The bounds are the values registered by vanilla's `RangedAttribute` entries
/// in `net.minecraft.world.entity.ai.attributes.Attributes`.
fn range(name: &str) -> (f64, f64) {
    match name {
        "air_drag_modifier" => (0.0, 2048.0),
        "armor" => (0.0, 30.0),
        "armor_toughness" => (0.0, 20.0),
        "attack_damage" => (0.0, 2048.0),
        "attack_knockback" => (0.0, 5.0),
        "attack_speed" => (0.0, 1024.0),
        "below_name_distance" => (0.0, 512.0),
        "block_break_speed" => (0.0, 1024.0),
        "block_interaction_range" => (0.0, 64.0),
        "bounciness" => (0.0, 1.0),
        "burning_time" => (0.0, 1024.0),
        "camera_distance" => (0.0, 32.0),
        "explosion_knockback_resistance" => (0.0, 1.0),
        "entity_interaction_range" => (0.0, 64.0),
        "fall_damage_multiplier" => (0.0, 100.0),
        "flying_speed" => (0.0, 1024.0),
        "follow_range" => (0.0, 2048.0),
        "friction_modifier" => (0.0, 2048.0),
        "gravity" => (-1.0, 1.0),
        "jump_strength" => (0.0, 32.0),
        "knockback_resistance" => (-2.0, 1.0),
        "luck" => (-1024.0, 1024.0),
        "max_absorption" => (0.0, 2048.0),
        "max_health" => (1.0, 1024.0),
        "mining_efficiency" => (0.0, 1024.0),
        "movement_efficiency" => (0.0, 1.0),
        "movement_speed" => (0.0, 1024.0),
        "name_tag_distance" => (0.0, 512.0),
        "oxygen_bonus" => (0.0, 1024.0),
        "safe_fall_distance" => (-1024.0, 1024.0),
        "scale" => (0.0625, 16.0),
        "sneaking_speed" => (0.0, 1.0),
        "spawn_reinforcements" => (0.0, 1.0),
        "step_height" => (0.0, 10.0),
        "submerged_mining_speed" => (0.0, 20.0),
        "sweeping_damage_ratio" => (0.0, 1.0),
        "tempt_range" => (0.0, 2048.0),
        "water_movement_efficiency" => (0.0, 1.0),
        "waypoint_transmit_range" | "waypoint_receive_range" => (0.0, 60_000_000.0),
        other => panic!("Missing vanilla range for attribute {other}"),
    }
}

/// Generates the `TokenStream` for the `Attributes` struct and its associated constants.
pub fn build() -> TokenStream {
    let attributes: BTreeMap<String, Attributes> =
        serde_json::from_str(&fs::read_to_string("../../assets/attributes.json").unwrap())
            .expect("Failed to parse attributes.json");

    let mut sorted_attributes: Vec<(String, Attributes)> = attributes.into_iter().collect();
    sorted_attributes.sort_by_key(|(_, raw)| raw.id);

    let mut constant_defs = Vec::new();
    let mut constant_idents = Vec::new();

    for (raw_name, raw_value) in sorted_attributes {
        let constant_ident = format_ident!("{}", raw_name.to_shouty_snake_case());
        constant_idents.push(constant_ident.clone());

        let id_lit = LitInt::new(&raw_value.id.to_string(), Span::call_site());
        let default_value_lit = raw_value.default_value;
        let (min_value, max_value) = range(&raw_name);
        let name_str = format!("minecraft:{raw_name}");

        constant_defs.push(quote!(
            pub const #constant_ident: Self = Self {
                id: #id_lit,
                default_value: #default_value_lit,
                min_value: #min_value,
                max_value: #max_value,
                name: #name_str,
            };
        ));
    }

    quote! {
        use std::hash::Hash;

        #[derive(Clone, Debug)]
        pub struct Attributes {
            pub id: u8,
            pub default_value: f64,
            pub min_value: f64,
            pub max_value: f64,
            pub name: &'static str,
        }
        impl PartialEq for Attributes {
            fn eq(&self, other: &Self) -> bool {
                self.id == other.id
            }
        }
        impl Eq for Attributes {}
        impl Hash for Attributes {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.id.hash(state);
            }
        }
        impl Attributes {
            #(#constant_defs)*

            pub const ALL: &'static [Self] = &[
                #(Self::#constant_idents),*
            ];
        }
    }
}
