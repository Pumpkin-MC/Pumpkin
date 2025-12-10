use std::{collections::BTreeMap, fs};

use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::TextContent::Translate;
use quote::{format_ident, quote};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Enchantment {
    pub id: u8,
    pub anvil_cost: u32,
    pub supported_items: String,
    pub description: TextComponent,
    pub exclusive_set: Option<String>,
    pub max_level: i32,
    pub slots: Vec<AttributeModifierSlot>, // TODO: add more
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

    let enchantments: BTreeMap<String, Enchantment> =
        serde_json::from_str(&fs::read_to_string("../assets/enchantments.json").unwrap())
            .expect("Failed to parse enchantments.json");

    let mut variants = TokenStream::new();
    let mut name_to_type = TokenStream::new();
    let mut id_to_type = TokenStream::new();

    for (name, enchantment) in enchantments.into_iter() {
        let id = enchantment.id;
        let raw_name = name.strip_prefix("minecraft:").unwrap();
        let format_name = format_ident!("{}", raw_name.to_shouty_snake_case());
        let anvil_cost = enchantment.anvil_cost;
        let supported_items = format_ident!(
            "{}",
            enchantment
                .supported_items
                .strip_prefix("#")
                .unwrap()
                .replace(":", "_")
                .replace("/", "_")
                .to_uppercase()
        );
        let max_level = enchantment.max_level;
        let slots = enchantment.slots;
        let slots = slots.iter().map(|slot| slot.to_tokens());
        let Translate { translate, with: _ } = &enchantment.description.0.content else {
            panic!()
        };
        let translate = translate.to_string();

        if let Some(exclusive_set) = &enchantment.exclusive_set {
            let exclusive_set = format_ident!(
                "{}",
                exclusive_set
                    .strip_prefix("#")
                    .unwrap()
                    .replace(":", "_")
                    .replace("/", "_")
                    .to_uppercase()
            );
            variants.extend([quote! {
                pub const #format_name: Self = Self {
                    id: #id,
                    name: #name,
                    registry_key: #raw_name,
                    description: #translate,
                    anvil_cost: #anvil_cost,
                    supported_items: &ItemTag::#supported_items,
                    exclusive_set: Some(&EnchantmentTag::#exclusive_set),
                    max_level: #max_level,
                    slots: &[#(#slots),*]
                };
            }]);
        } else {
            variants.extend([quote! {
                pub const #format_name: Self = Self {
                    id: #id,
                    name: #name,
                    description: #translate,
                    registry_key: #raw_name,
                    anvil_cost: #anvil_cost,
                    supported_items: &ItemTag::#supported_items,
                    exclusive_set: None,
                    max_level: #max_level,
                    slots: &[#(#slots),*]
                };
            }]);
        }

        name_to_type.extend(quote! { #name => Some(&Self::#format_name), });
        id_to_type.extend(quote! { #id => Some(&Self::#format_name), });
    }

    quote! {
        use crate::item::Item;
        use crate::tag::Enchantment as EnchantmentTag;
        use crate::tag::Item as ItemTag;
        use crate::tag::{RegistryKey, Tag, Taggable};
        use crate::data_component_impl::EnchantmentsImpl;
        use pumpkin_util::text::TextComponent;
        use pumpkin_util::text::color::NamedColor;
        use std::hash::{Hash, Hasher};
        use std::slice::Iter;

        pub struct Enchantment {
            pub id: u8,
            pub name: &'static str,
            pub registry_key: &'static str,
            pub description: &'static str, // TODO use TextComponent
            pub anvil_cost: u32,
            pub supported_items: &'static Tag,
            pub exclusive_set: Option<&'static Tag>,
            pub max_level: i32,
            pub slots: &'static [AttributeModifierSlot]
            // TODO: add more
        }
        impl Taggable for Enchantment {
            #[inline]
            fn tag_key() -> RegistryKey {
                RegistryKey::Enchantment
            }
            #[inline]
            fn registry_key(&self) -> &str {
                self.registry_key
            }
            #[inline]
            fn registry_id(&self) -> u16 {
                self.id as u16
            }
        }
        impl PartialEq for Enchantment {
            fn eq(&self, other: &Self) -> bool {
                self.id == other.id
            }
        }
        impl Eq for Enchantment {}
        impl Hash for Enchantment {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.id.hash(state);
            }
        }
        #[derive(Debug, Clone, Hash, PartialEq)]
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

            pub fn from_name(name: &str) -> Option<&'static Self> {
                match name {
                    #name_to_type
                    _ => None
                }
            }
            pub fn from_id(id: u8) -> Option<&'static Self> {
                match id {
                    #id_to_type
                    _ => None
                }
            }

            pub fn can_enchant(&self, item: &'static Item) -> bool {
                self.supported_items.1.contains(&item.id)
            }
            pub fn are_compatible(&self, other: &'static Enchantment) -> bool {
                if self == other {
                    return false;
                }
                if let Some(tag) = self.exclusive_set && tag.1.contains(&(other.id as u16)) {
                    return false;
                }
                if let Some(tag) = other.exclusive_set && tag.1.contains(&(self.id as u16)) {
                    return false;
                }
                true
            }
            pub fn is_enchantment_compatible(&self, other: &EnchantmentsImpl) -> bool {
                for (i, _) in other.enchantment.iter() {
                    if !self.are_compatible(i) {
                        return false;
                    }
                }
                true
            }
            pub fn get_fullname(&self, level: i32) -> TextComponent {
                let mut ret = TextComponent::translate(self.description, []).color_named(
                    if self.has_tag(&EnchantmentTag::MINECRAFT_CURSE) {
                        NamedColor::Red
                    } else {
                        NamedColor::Gray
                    }
                );
                if level != 1 || self.max_level != 1 {
                    ret = ret.add_text(" ")
                        .add_child(TextComponent::translate("enchantment.level.".to_string() + &level.to_string(), []));
                }
                ret
            }
        }
    }
}
