use std::{collections::HashMap, fs};

use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

#[derive(Deserialize)]
struct Enchantment {
    anvil_cost: u32,
    supported_items: String,
    max_level: i32,
    slots: Vec<AttributeModifierSlot>, // TODO: add more
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum AttributeModifierSlot {
    Any,
    MainHand,
    OffHand,
    Hand,
    Feet,
    Legs,
    Chest,
    Head,
    Armor,
    Body,
    Saddle,
}

impl AttributeModifierSlot {
    pub fn to_tokens(&self) -> TokenStream {
        match self {
            AttributeModifierSlot::Any => quote! { AttributeModifierSlot::Any },
            AttributeModifierSlot::MainHand => quote! { AttributeModifierSlot::MainHand },
            AttributeModifierSlot::OffHand => quote! { AttributeModifierSlot::OffHand },
            AttributeModifierSlot::Hand => quote! { AttributeModifierSlot::Hand },
            AttributeModifierSlot::Feet => quote! { AttributeModifierSlot::Feet },
            AttributeModifierSlot::Legs => quote! { AttributeModifierSlot::Legs },
            AttributeModifierSlot::Chest => quote! { AttributeModifierSlot::Chest },
            AttributeModifierSlot::Head => quote! { AttributeModifierSlot::Head },
            AttributeModifierSlot::Armor => quote! { AttributeModifierSlot::Armor },
            AttributeModifierSlot::Body => quote! { AttributeModifierSlot::Body },
            AttributeModifierSlot::Saddle => quote! { AttributeModifierSlot::Saddle },
        }
    }
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/enchantments.json");

    let enchantments: HashMap<String, Enchantment> =
        serde_json::from_str(&fs::read_to_string("../assets/enchantments.json").unwrap())
            .expect("Failed to parse biome.json");

    let mut variants = TokenStream::new();
    let mut name_to_type = TokenStream::new();

    for (name, enchantment) in enchantments.iter() {
        let raw_name = name.strip_prefix("minecraft:").unwrap();
        let format_name = format_ident!("{}", raw_name.to_shouty_snake_case());
        let anvil_cost = enchantment.anvil_cost;
        let supported_items = enchantment.supported_items.clone();
        let max_level = enchantment.max_level;
        let slots = enchantment.slots.clone();
        let slots = slots.iter().map(|slot| slot.to_tokens());

        variants.extend([quote! {
            pub const #format_name: Enchantment = Enchantment {
               name: #name,
               anvil_cost: #anvil_cost,
               supported_items: #supported_items,
               max_level: #max_level,
               slots: &[#(#slots),*]
            };
        }]);

        name_to_type.extend(quote! { #name => Some(Self::#format_name), });
    }

    quote! {
        use std::hash::Hash;
        #[derive(Debug, Clone)]
        pub struct Enchantment {
            pub name: &'static str,
            pub anvil_cost: u32,
            pub supported_items: &'static str,
            pub max_level: i32,
            pub slots: &'static [AttributeModifierSlot]
            // TODO: add more
        }

        #[derive(Debug, Clone, Hash)]
        pub enum AttributeModifierSlot {
            Any,
            MainHand,
            OffHand,
            Hand,
            Feet,
            Legs,
            Chest,
            Head,
            Armor,
            Body,
            Saddle,
        }

        impl Enchantment {
            #variants

            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    #name_to_type
                    _ => None
                }
            }
        }
    }
}
