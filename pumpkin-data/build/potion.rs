use std::{collections::BTreeMap, fs};

use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

#[derive(Deserialize)]
struct Potion {
    id: u8,
    effects: Vec<Effect>,
}

#[derive(Deserialize)]
pub struct Effect {
    effect_type: String,
    duration: i32,
    amplifier: u8,
    ambient: bool,
    show_particles: bool,
    show_icon: bool,
}

impl Effect {
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

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/potion.json");

    let potions: BTreeMap<String, Potion> =
        serde_json::from_str(&fs::read_to_string("../assets/potion.json").unwrap())
            .expect("Failed to parse potion.json");

    let mut variants = TokenStream::new();
    let mut name_to_type = TokenStream::new();

    for (name, potion) in potions.into_iter() {
        let format_name = format_ident!("{}", name.to_shouty_snake_case());
        let id = potion.id;
        let slots = potion.effects;
        let slots = slots.iter().map(|slot| slot.to_tokens());

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
