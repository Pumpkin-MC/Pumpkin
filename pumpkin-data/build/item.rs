use crate::enchantments::AttributeModifierSlot;
use heck::ToShoutySnakeCase;
use proc_macro2::{Span, TokenStream};
use pumpkin_util::registry::TagType;
use pumpkin_util::text::TextContent;
use pumpkin_util::{registry::RegistryEntryList, text::TextComponent};
use quote::{ToTokens, format_ident, quote};
use serde::Deserialize;
use std::{collections::BTreeMap, fs};
use syn::{Ident, LitBool, LitFloat, LitInt, LitStr};

#[derive(Deserialize)]
pub struct Item {
    pub id: u16,
    pub components: ItemComponents,
}

#[derive(Deserialize)]
pub struct ItemComponents {
    #[serde(rename = "minecraft:item_name")]
    pub item_name: TextComponent,
    #[serde(rename = "minecraft:max_stack_size")]
    pub max_stack_size: u8,
    #[serde(rename = "minecraft:jukebox_playable")]
    pub jukebox_playable: Option<String>,
    #[serde(rename = "minecraft:damage")]
    pub damage: Option<u16>,
    #[serde(rename = "minecraft:max_damage")]
    pub max_damage: Option<u16>,
    #[serde(rename = "minecraft:attribute_modifiers")]
    pub attribute_modifiers: Option<Vec<Modifier>>,
    #[serde(rename = "minecraft:tool")]
    pub tool: Option<ToolComponent>,
    #[serde(rename = "minecraft:food")]
    pub food: Option<FoodComponent>,
    #[serde(rename = "minecraft:equippable")]
    pub equippable: Option<EquippableComponent>,
    #[serde(rename = "minecraft:consumable")]
    pub consumable: Option<Consumable>,
    #[serde(rename = "minecraft:blocks_attacks")]
    pub blocks_attacks: Option<BlocksAttacks>,
    #[serde(rename = "minecraft:death_protection")]
    pub death_protection: Option<DeathProtection>,
}

impl ToTokens for ItemComponents {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let max_stack_size = LitInt::new(&self.max_stack_size.to_string(), Span::call_site());
        tokens.extend(quote! {
            (MaxStackSize, &MaxStackSizeImpl {
                size: #max_stack_size,
            }),
        });
        if let Some(playable) = &self.jukebox_playable {
            let song = LitStr::new(playable, Span::call_site());
            tokens.extend(quote! {
                (JukeboxPlayable, &JukeboxPlayableImpl{
                    song: #song,
                }),
            });
        }

        let TextContent::Translate {
            translate: text,
            with: _,
        } = self.item_name.clone().0.content
        else {
            unreachable!()
        };
        let item_name = LitStr::new(&text, Span::call_site());
        tokens.extend(quote! {
            (ItemName, &ItemNameImpl {
                name: #item_name,
            }),
        });

        if let Some(d) = self.damage {
            let damage_lit = LitInt::new(&d.to_string(), Span::call_site());
            tokens.extend(quote! {
                (Damage, &DamageImpl {
                    damage: #damage_lit,
                }),
            });
        };

        if let Some(md) = self.max_damage {
            let max_damage_lit = LitInt::new(&md.to_string(), Span::call_site());
            tokens.extend(quote! {
                (MaxDamage, &MaxDamageImpl {
                    max_damage: #max_damage_lit,
                }),
            });
        };

        if let Some(modifiers) = &self.attribute_modifiers {
            let modifier_code = modifiers.iter().map(|modifier| {
                let r#type = format_ident!(
                    "{}",
                    modifier
                        .r#type
                        .strip_prefix("minecraft:")
                        .unwrap()
                        .to_uppercase()
                );
                let id = LitStr::new(&modifier.id, Span::call_site());
                let amount = modifier.amount;
                let operation = Ident::new(&format!("{:?}", modifier.operation), Span::call_site());
                let slot = modifier.slot.to_tokens();

                quote! {
                    Modifier {
                        r#type: &Attributes::#r#type,
                        id: #id,
                        amount: #amount,
                        operation: Operation::#operation,
                        slot: #slot,
                    }
                }
            });
            tokens.extend(quote! {
                (AttributeModifiers, &AttributeModifiersImpl {
                    attribute_modifiers: Cow::Borrowed(&[#(#modifier_code),*])
                }),
            });
        };

        if let Some(tool) = &self.tool {
            let rules_code = tool.rules.iter().map(|rule| {
                let block_array;

                if let RegistryEntryList::Single(t) = &rule.blocks {
                    if let TagType::Item(str) = t {
                        let ident = format_ident!(
                            "{}",
                            str.strip_prefix("minecraft:").unwrap().to_uppercase()
                        );
                        block_array = quote! {
                            Blocks(Cow::Borrowed(&[&Block::#ident]))
                        }
                    } else if let TagType::Tag(str) = t {
                        let ident = format_ident!(
                            "{}",
                            str.replace(":", "_").replace("/", "_").to_uppercase()
                        );
                        block_array = quote! {
                            Tag(&tag::Block::#ident)
                        }
                    } else {
                        unreachable!();
                    }
                } else if let RegistryEntryList::Many(t) = &rule.blocks {
                    let mut array = vec![];
                    for i in t {
                        let TagType::Item(str) = i else {
                            unreachable!();
                        };
                        let ident = format_ident!(
                            "{}",
                            str.strip_prefix("minecraft:").unwrap().to_uppercase()
                        );
                        array.push(quote! {
                            &Block::#ident
                        });
                    }
                    block_array = quote! {
                        Blocks(Cow::Borrowed(&[#(#array),*]))
                    }
                } else {
                    unreachable!();
                }
                let speed = match rule.speed {
                    Some(speed) => {
                        quote! { Some(#speed) }
                    }
                    None => quote! { None },
                };
                let correct_for_drops = match rule.correct_for_drops {
                    Some(correct_for_drops) => {
                        let correct_for_drops = LitBool::new(correct_for_drops, Span::call_site());
                        quote! { Some(#correct_for_drops) }
                    }
                    None => quote! { None },
                };
                quote! {
                    ToolRule {
                        blocks: #block_array,
                        speed: #speed,
                        correct_for_drops: #correct_for_drops
                    }
                }
            });
            let damage_per_block = {
                let speed = LitInt::new(&tool.damage_per_block.to_string(), Span::call_site());
                quote! { #speed }
            };
            let default_mining_speed = {
                let speed = LitFloat::new(
                    &format!("{:.1}", tool.default_mining_speed),
                    Span::call_site(),
                );
                quote! { #speed }
            };
            let can_destroy_blocks_in_creative =
                LitBool::new(tool.can_destroy_blocks_in_creative, Span::call_site());
            tokens.extend(quote! { (Tool, &ToolImpl {
                rules: Cow::Borrowed(&[#(#rules_code),*]),
                default_mining_speed: #default_mining_speed,
                damage_per_block: #damage_per_block,
                can_destroy_blocks_in_creative: #can_destroy_blocks_in_creative
            }), });
        };

        if let Some(food) = &self.food {
            let nutrition = LitInt::new(&food.nutrition.to_string(), Span::call_site());
            let saturation = LitFloat::new(&format!("{:.1}", food.saturation), Span::call_site());
            let can_always_eat = {
                let can = LitBool::new(food.can_always_eat, Span::call_site());
                quote! { #can }
            };
            tokens.extend(quote! { (Food, &FoodImpl {
                nutrition: #nutrition,
                saturation: #saturation,
                can_always_eat: #can_always_eat,
            }), });
        };

        if let Some(consumable) = &self.consumable {
            let consume_seconds = LitFloat::new(
                &format!("{:.1}", consumable.consume_seconds.unwrap_or(1.6)),
                Span::call_site(),
            );

            tokens.extend(quote! { (Consumable, &ConsumableImpl {
                consume_seconds: #consume_seconds,
            }), });
        };

        if self.blocks_attacks.is_some() {
            tokens.extend(quote! { (BlocksAttacks, &BlocksAttacksImpl), });
        };

        if self.death_protection.is_some() {
            tokens.extend(quote! { (DeathProtection, &DeathProtectionImpl), });
        };

        if let Some(equippable) = &self.equippable {
            let slot = match equippable.slot.as_str() {
                "mainhand" => quote! { &EquipmentSlot::MAIN_HAND },
                "offhand" => quote! { &EquipmentSlot::OFF_HAND },
                "head" => quote! { &EquipmentSlot::HEAD },
                "chest" => quote! { &EquipmentSlot::CHEST },
                "legs" => quote! { &EquipmentSlot::LEGS },
                "feet" => quote! { &EquipmentSlot::FEET },
                "body" => quote! { &EquipmentSlot::BODY },
                "saddle" => quote! { &EquipmentSlot::SADDLE },
                _ => panic!("Unknown equippable slot: {}", equippable.slot),
            };
            let equip_sound = equippable
                .equip_sound
                .as_ref()
                .map(|s| {
                    let equip_sound = LitStr::new(s, Span::call_site());
                    quote! { #equip_sound }
                })
                .unwrap_or(quote! { "item.armor.equip_generic" });
            let asset_id = equippable
                .asset_id
                .as_ref()
                .map(|s| {
                    let asset_id = LitStr::new(s, Span::call_site());
                    quote! { Some(#asset_id) }
                })
                .unwrap_or(quote! { None });
            let camera_overlay = equippable
                .camera_overlay
                .as_ref()
                .map(|s| {
                    let camera_overlay = LitStr::new(s, Span::call_site());
                    quote! { Some(#camera_overlay) }
                })
                .unwrap_or(quote! { None });
            let allowed_entities = equippable
                .allowed_entities
                .clone()
                .map(|list| {
                    let vec: Vec<_> = list
                        .into_vec()
                        .iter()
                        .map(|reg| {
                            match reg {
                                TagType::Item(item) => {
                                    let ident = format_ident!(
                                        "{}",
                                        item.strip_prefix("minecraft:").unwrap().to_uppercase()
                                    );
                                    quote! { EntityTypeOrTag::Single(&crate::entity_type::EntityType::#ident) }
                                },
                                TagType::Tag(tag) => {
                                    let ident = format_ident!(
                                        "{}",
                                        tag.replace(":", "_").replace("/", "_").to_uppercase()
                                    );
                                    quote! { EntityTypeOrTag::Tag(&crate::tag::EntityType::#ident) }
                                }
                            }
                        })
                        .collect();
                    quote! {
                        Some(&[#(#vec),*])
                    }
                })
                .unwrap_or(quote! { None });
            let dispensable = LitBool::new(equippable.dispensable, Span::call_site());
            let swappable = LitBool::new(equippable.swappable, Span::call_site());
            let damage_on_hurt = LitBool::new(equippable.damage_on_hurt, Span::call_site());
            let equip_on_interact = LitBool::new(equippable.equip_on_interact, Span::call_site());
            let can_be_sheared = LitBool::new(equippable.can_be_sheared, Span::call_site());
            let shearing_sound = equippable
                .shearing_sound
                .as_ref()
                .map(|s| {
                    let shearing_sound = LitStr::new(s, Span::call_site());
                    quote! {
                        Some(#shearing_sound)
                    }
                })
                .unwrap_or(quote! { None });

            tokens.extend(quote! { (Equippable, &EquippableImpl {
                slot: #slot,
                equip_sound: #equip_sound,
                asset_id: #asset_id,
                camera_overlay: #camera_overlay,
                allowed_entities: #allowed_entities,
                dispensable: #dispensable,
                swappable: #swappable,
                damage_on_hurt: #damage_on_hurt,
                equip_on_interact: #equip_on_interact,
                can_be_sheared: #can_be_sheared,
                shearing_sound: #shearing_sound
            }), });
        };
    }
}

fn return_1u32() -> u32 {
    1
}

fn return_1f32() -> f32 {
    1.
}

fn return_true() -> bool {
    true
}
#[derive(Deserialize)]
pub struct ToolComponent {
    rules: Vec<ToolRule>,
    #[serde(default = "return_1f32")]
    default_mining_speed: f32,
    #[serde(default = "return_1u32")]
    damage_per_block: u32,
    #[serde(default = "return_true")]
    can_destroy_blocks_in_creative: bool,
}

fn return_false() -> bool {
    false
}

#[derive(Deserialize, Copy, Clone)]
pub struct FoodComponent {
    nutrition: u8,
    saturation: f32,
    #[serde(default = "return_false")]
    can_always_eat: bool,
}

#[derive(Deserialize, Clone)]
pub struct ToolRule {
    blocks: RegistryEntryList,
    speed: Option<f32>,
    correct_for_drops: Option<bool>,
}

#[derive(Deserialize, Clone)]
pub struct Modifier {
    pub r#type: String,
    pub id: String,
    pub amount: f64,
    pub operation: Operation,
    // TODO: Make this an enum
    pub slot: AttributeModifierSlot,
}

fn _true() -> bool {
    true
}

#[derive(Deserialize, Clone)]
pub struct Consumable {
    consume_seconds: Option<f32>, // TODO
}

#[derive(Deserialize, Clone)]
pub struct DeathProtection {
    // TODO
}

#[derive(Deserialize, Clone)]
pub struct BlocksAttacks {
    // TODO
}

#[allow(dead_code)]
#[derive(Deserialize, Clone)]
pub struct EquippableComponent {
    pub slot: String,
    pub equip_sound: Option<String>,
    pub asset_id: Option<String>,
    pub camera_overlay: Option<String>,
    pub allowed_entities: Option<RegistryEntryList>,
    #[serde(default = "_true")]
    pub dispensable: bool,
    #[serde(default = "_true")]
    pub swappable: bool,
    #[serde(default = "_true")]
    pub damage_on_hurt: bool,
    #[serde(default)]
    pub equip_on_interact: bool,
    #[serde(default)]
    pub can_be_sheared: bool,
    pub shearing_sound: Option<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)]
pub enum Operation {
    AddValue,
    AddMultipliedBase,
    AddMultipliedTotal,
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/items.json");

    let items: BTreeMap<String, Item> =
        serde_json::from_str(&fs::read_to_string("../assets/items.json").unwrap())
            .expect("Failed to parse items.json");

    let mut type_from_raw_id_arms = TokenStream::new();
    let mut type_from_name = TokenStream::new();

    let mut constants = TokenStream::new();

    for (name, item) in items {
        let const_ident = format_ident!("{}", name.to_shouty_snake_case());

        let components = &item.components;
        let components_tokens = components.to_token_stream();

        let id_lit = LitInt::new(&item.id.to_string(), Span::call_site());

        constants.extend(quote! {
            pub const #const_ident: Item = Item {
                id: #id_lit,
                registry_key: #name,
                components: &[#components_tokens],
            };
        });

        type_from_raw_id_arms.extend(quote! {
            #id_lit => Some(&Self::#const_ident),
        });

        type_from_name.extend(quote! {
            #name => Some(&Self::#const_ident),
        });
    }

    quote! {
        use crate::data_component::DataComponent::*;
        use crate::data_component_impl::*;
        use crate::tag::{RegistryKey, Taggable};
        use pumpkin_util::text::TextComponent;
        use std::borrow::Cow;
        use std::hash::{Hash, Hasher};
        use crate::{tag, AttributeModifierSlot};
        use crate::attributes::Attributes;
        use crate::data_component_impl::IDSet::{Blocks, Tag};
        use crate::data_component::DataComponent;
        use crate::Block;

        #[derive(Clone)]
        pub struct Item {
            pub id: u16,
            pub registry_key: &'static str,
            pub components: &'static [(DataComponent, &'static dyn DataComponentImpl)],
        }

        impl PartialEq for Item {
            fn eq(&self, other: &Self) -> bool {
                self.id == other.id
            }
        }

        impl Eq for Item {}

        impl Hash for Item {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.id.hash(state);
            }
        }

        impl Item {
            #constants

            pub fn translated_name(&self) -> TextComponent {
                TextComponent::translate(
                    self.components
                        .iter()
                        .find_map(|(id, data)| if id == &ItemName {
                            Some(data.as_any().downcast_ref::<ItemNameImpl>().unwrap().name)
                        } else {
                            None
                        }
                    ).unwrap(),
                    &[],
                )
            }

            #[doc = "Try to parse an item from a resource location string."]
            pub fn from_registry_key(name: &str) -> Option<&'static Self> {
                let name = name.strip_prefix("minecraft:").unwrap_or(name);
                match name {
                    #type_from_name
                    _ => None
                }
            }

            #[doc = "Try to parse an item from a raw id."]
            pub const fn from_id(id: u16) -> Option<&'static Self> {
                match id {
                    #type_from_raw_id_arms
                    _ => None
                }
            }
        }

        impl Taggable for Item {
            #[inline]
            fn tag_key() -> RegistryKey {
                RegistryKey::Item
            }

            #[inline]
            fn registry_key(&self) -> &str {
                self.registry_key
            }

            #[inline]
            fn registry_id(&self) -> u16 {
                self.id
            }
        }
    }
}
