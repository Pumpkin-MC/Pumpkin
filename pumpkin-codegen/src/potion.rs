use std::{collections::BTreeMap, fs};

use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

/// Raw deserialization shape for a single potion entry from `potion.json`.
#[derive(Deserialize)]
struct Potion {
    /// Numeric registry ID for this potion.
    id: u8,
    /// Status effects granted when this potion is consumed or applied.
    effects: Vec<Effect>,
}

/// A single status effect instance applied by a potion, as defined in `potion.json`.
#[derive(Deserialize)]
pub struct Effect {
    /// Namespaced effect resource location (e.g. `"minecraft:speed"`).
    effect_type: String,
    /// Duration of the effect in ticks.
    duration: i32,
    /// Amplifier level (0 = level I, 1 = level II, …).
    amplifier: u8,
    /// Whether this effect is ambient (produced by beacon, reducing particle density).
    ambient: bool,
    /// Whether particles should be displayed while the effect is active.
    show_particles: bool,
    /// Whether the effect icon should appear in the HUD.
    show_icon: bool,
}

impl Effect {
    /// Converts this effect entry into a `TokenStream` for use in generated code.
    pub fn to_tokens(&self) -> TokenStream {
        let effect_type = format_ident!(
            "{}",
            self.effect_type
                .strip_prefix("minecraft:")
                .unwrap()
                .to_uppercase()
        );
        let duration = self.duration;
        let amplifier = self.amplifier;
        let ambient = self.ambient;
        let show_particles = self.show_particles;
        let show_icon = self.show_icon;
        quote! {
            Effect {
                effect_type: &StatusEffect::#effect_type,
                duration: #duration,
                amplifier: #amplifier,
                ambient: #ambient,
                show_particles: #show_particles,
                show_icon: #show_icon,
                blend: false,
            }
        }
    }
}

/// Generates the `TokenStream` for the `Potion` struct, `Effect` struct, and `from_name` lookup.
pub fn build() -> TokenStream {
    let potions: BTreeMap<String, Potion> =
        serde_json::from_str(&fs::read_to_string("../assets/potion.json").unwrap())
            .expect("Failed to parse potion.json");

    let mut variants = TokenStream::new();
    let mut name_to_type = TokenStream::new();

    for (name, potion) in potions {
        let format_name = format_ident!("{}", name.to_shouty_snake_case());
        let id = potion.id;
        let slots = potion.effects;
        let slots = slots.iter().map(Effect::to_tokens);

        variants.extend([quote! {
            pub const #format_name: Self = Self {
                name: #name,
                id: #id,
                effects: &[#(#slots),*],
            };
        }]);

        name_to_type.extend(quote! { #name => Some(&Self::#format_name), });
    }

    quote! {
        use std::hash::Hash;
        use crate::effect::StatusEffect;

        pub struct Potion {
            pub id: u8,
            pub name: &'static str,
            pub effects: &'static [Effect],
        }

        #[derive(Clone)]
        pub struct Effect {
            pub effect_type: &'static StatusEffect,
            pub duration: i32,
            pub amplifier: u8,
            pub ambient: bool,
            pub show_particles: bool,
            pub show_icon: bool,
            pub blend: bool,
        }

        impl Potion {
            #variants

            pub fn from_name(name: &str) -> Option<&'static Self> {
                match name {
                    #name_to_type
                    _ => None
                }
            }
        }
    }
}
