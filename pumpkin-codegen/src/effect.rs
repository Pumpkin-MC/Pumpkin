use std::{collections::BTreeMap, fs};

use heck::{ToPascalCase, ToShoutySnakeCase};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

/// Raw deserialization shape for a single mob effect entry from `effect.json`.
#[derive(Deserialize)]
struct Effect {
    /// Numeric registry ID for this mob effect.
    id: u8,
    /// Category controlling UI presentation and general behavior.
    category: MobEffectCategory,
    /// Particle and icon color for this effect as a packed RGB integer.
    color: i32,
    /// Translation key used to display the effect name in the UI.
    translation_key: String,
    /// Attribute modifiers applied to entities affected by this effect.
    attribute_modifiers: Vec<Modifiers>,
}

/// Classification of a mob effect determining its UI presentation and general purpose.
#[expect(clippy::upper_case_acronyms)]
#[derive(Deserialize)]
pub enum MobEffectCategory {
    BENEFICIAL,
    HARMFUL,
    NEUTRAL,
}

/// A single attribute modifier applied by a mob effect, as defined in `effect.json`.
#[derive(Deserialize)]
pub struct Modifiers {
    /// Namespaced attribute resource location (e.g. `"minecraft:generic.attack_damage"`).
    attribute: String,
    /// Unique resource location ID for this modifier (used for stacking prevention).
    id: String,
    /// The base numeric value applied by this modifier.
    #[serde(rename = "baseValue")]
    base_value: f64,
    /// The operation used to combine this modifier with the base attribute value.
    operation: String,
}

impl Modifiers {
    /// Converts this modifier entry into a `TokenStream` for use in generated code.
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
    /// Converts this category variant into a `TokenStream` for use in generated code.
    pub fn to_tokens(&self) -> TokenStream {
        match self {
            Self::BENEFICIAL => quote! { MobEffectCategory::Beneficial },
            Self::HARMFUL => quote! { MobEffectCategory::Harmful },
            Self::NEUTRAL => quote! { MobEffectCategory::Neutral },
        }
    }
}

/// Generates the `TokenStream` for the `StatusEffect` struct, `MobEffectCategory`, `Modifiers`,
/// and the `from_name`/`from_minecraft_name` lookup methods.
pub fn build() -> TokenStream {
    let effects: BTreeMap<String, Effect> =
        serde_json::from_str(&fs::read_to_string("../assets/effect.json").unwrap())
            .expect("Failed to parse effect.json");

    let mut variants = TokenStream::new();
    let mut name_to_type = TokenStream::new();
    let mut minecraft_name_to_type = TokenStream::new();
    let mut id_to_type = TokenStream::new();

    for (name, effect) in effects {
        let format_name = format_ident!("{}", name.to_shouty_snake_case());
        let id = effect.id;
        let u16_id = effect.id as u16;
        let color = effect.color;
        let translation_key = effect.translation_key;
        let category = effect.category.to_tokens();
        let slots = effect.attribute_modifiers;
        let slots = slots.into_iter().map(Modifiers::get_tokens);

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
        id_to_type.extend(quote! { #u16_id => Some(&Self::#format_name), })
    }

    quote! {
        use std::hash::{Hash, Hasher};
        use crate::{attributes::Attributes, data_component_impl::{Operation, IDSetContent}};
        #[derive(Clone, Debug)]
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
                    #minecraft_name_to_type
                    _ => None
                }
            }
        }
        impl IDSetContent for StatusEffect {
            fn registry_id(&self) -> u16 {
                self.id as u16
            }

            fn from_id(id: u16) -> Option<&'static Self> {
                match id {
                    #id_to_type
                    _ => None
                }
            }

            fn from_str(name: &str) -> Option<&'static Self> {
                StatusEffect::from_minecraft_name(name)
            }

            fn to_string(&self) -> String {
                self.minecraft_name.to_string()
            }
        }
    }
}
