use std::{collections::BTreeMap, fs};

use heck::{ToPascalCase, ToShoutySnakeCase};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

#[derive(Deserialize)]
struct Effect {
    id: u8,
    category: MobEffectCategory,
    color: i32,
    translation_key: String,
    attribute_modifiers: Vec<Modifiers>,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Deserialize)]
pub enum MobEffectCategory {
    BENEFICIAL,
    HARMFUL,
    NEUTRAL,
}

#[derive(Deserialize)]
pub struct Modifiers {
    attribute: String,
    id: String,
    #[serde(rename = "baseValue")]
    base_value: f64,
    operation: String,
}

impl Modifiers {
    pub fn get_tokens(self) -> TokenStream {
        let attribute = format_ident!("{}", self.attribute.to_uppercase());
        let id = self.id;
        let base_value = self.base_value;
        let operation = format_ident!("{}", self.operation.to_pascal_case());
        quote! {
            Modifiers {
                attribute: &Attributes::#attribute,
                id: #id,
                base_value: #base_value,
                operation: Operation::#operation,
            }
        }
    }
}

impl MobEffectCategory {
    pub fn to_tokens(&self) -> TokenStream {
        match self {
            MobEffectCategory::BENEFICIAL => quote! { MobEffectCategory::Beneficial },
            MobEffectCategory::HARMFUL => quote! { MobEffectCategory::Harmful },
            MobEffectCategory::NEUTRAL => quote! { MobEffectCategory::Neutral },
        }
    }
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/effect.json");

    let effects: BTreeMap<String, Effect> =
        serde_json::from_str(&fs::read_to_string("../assets/effect.json").unwrap())
            .expect("Failed to parse effect.json");

    let mut variants = TokenStream::new();
    let mut name_to_type = TokenStream::new();
    let mut minecraft_name_to_type = TokenStream::new();

    for (name, effect) in effects.into_iter() {
        let format_name = format_ident!("{}", name.to_shouty_snake_case());
        let id = effect.id;
        let color = effect.color;
        let translation_key = effect.translation_key;
        let category = effect.category.to_tokens();
        let slots = effect.attribute_modifiers;
        let slots = slots.into_iter().map(|slot| slot.get_tokens());

        let minecraft_name = "minecraft:".to_string() + &name;
        variants.extend([quote! {
            pub const #format_name: Self = Self {
                minecraft_name: #minecraft_name,
                id: #id,
                category: #category,
                color: #color,
                translation_key: #translation_key,
                attribute_modifiers: &[#(#slots),*],
            };
        }]);

        name_to_type.extend(quote! { #name => Some(&Self::#format_name), });

        minecraft_name_to_type.extend(quote! { #minecraft_name => Some(&Self::#format_name), });
    }

    quote! {
        use std::hash::{Hash, Hasher};
        use crate::attributes::Attributes;
        use crate::data_component_impl::Operation;

        pub struct StatusEffect {
            pub minecraft_name: &'static str,
            pub id: u8,
            pub category: MobEffectCategory,
            pub color: i32,
            pub translation_key: &'static str,
            pub attribute_modifiers: &'static [Modifiers],
        }

        impl PartialEq for StatusEffect {
            fn eq(&self, other: &Self) -> bool {
                self.id == other.id
            }
        }

        impl Eq for StatusEffect {}

        impl Hash for StatusEffect {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.id.hash(state);
            }
        }

        #[derive(Debug, Clone, Hash)]
        pub enum MobEffectCategory {
            Beneficial,
            Harmful,
            Neutral,
        }

        #[derive(Debug)]
        pub struct Modifiers {
            pub attribute: &'static Attributes,
            pub id: &'static str,
            pub base_value: f64,
            pub operation: Operation,
        }

        impl StatusEffect {
            #variants

            pub fn from_name(name: &str) -> Option<&'static Self> {
                match name {
                    #name_to_type
                    _ => None
                }
            }
            pub fn from_minecraft_name(name: &str) -> Option<&'static Self> {
                match name {
                    #name_to_type
                    _ => None
                }
            }
        }
    }
}
