use std::{collections::HashMap, fs};

use heck::ToShoutySnakeCase;
use proc_macro2::{Span, TokenStream};
use pumpkin_util::{registry::RegistryEntryList, text::TextComponent};
use quote::{ToTokens, format_ident, quote};
use serde::Deserialize;
use syn::{Ident, LitBool, LitFloat, LitInt, LitStr};

#[derive(Deserialize, Clone, Debug)]
pub struct Item {
    pub id: u16,
    pub components: ItemComponents,
}

#[derive(Deserialize, Clone, Debug)]
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
}

impl ToTokens for ItemComponents {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let max_stack_size = LitInt::new(&self.max_stack_size.to_string(), Span::call_site());
        let jukebox_playable = match &self.jukebox_playable {
            Some(playable) => {
                let song = LitStr::new(playable, Span::call_site());
                quote! { Some(#song) }
            }
            None => quote! { None },
        };

        let item_name = {
            let text = self.item_name.clone().get_text();
            let item_name = LitStr::new(&text, Span::call_site());
            quote! { #item_name }
        };

        let damage = match self.damage {
            Some(d) => {
                let damage_lit = LitInt::new(&d.to_string(), Span::call_site());
                quote! { Some(#damage_lit) }
            }
            None => quote! { None },
        };

        let max_damage = match self.max_damage {
            Some(md) => {
                let max_damage_lit = LitInt::new(&md.to_string(), Span::call_site());
                quote! { Some(#max_damage_lit) }
            }
            None => quote! { None },
        };

        let attribute_modifiers = match &self.attribute_modifiers {
            Some(modifiers) => {
                let modifier_code = modifiers.iter().map(|modifier| {
                    let r#type = LitStr::new(&modifier.r#type, Span::call_site());
                    let id = LitStr::new(&modifier.id, Span::call_site());
                    let amount = modifier.amount;
                    let operation =
                        Ident::new(&format!("{:?}", modifier.operation), Span::call_site());
                    let slot = LitStr::new(&modifier.slot, Span::call_site());

                    quote! {
                        Modifier {
                            r#type: #r#type,
                            id: #id,
                            amount: #amount,
                            operation: Operation::#operation,
                            slot: #slot,
                        }
                    }
                });
                quote! { Some(&[#(#modifier_code),*]) }
            }
            None => quote! { None },
        };

        let tool = match &self.tool {
            Some(tool) => {
                let rules_code = tool.rules.iter().map(|rule| {
                    let mut block_array = Vec::new();

                    // TODO: According to the wiki, this can be a string or a list.
                    // I dont think there'll be any issues with always using a list, but we can
                    // probably save bandwidth by doing single strings.
                    for reg in rule.blocks.get_values() {
                        let tag_string = reg.serialize();
                        // The client knows what tags are; just send them the tag instead of all the
                        // blocks that are a part of the tag.
                        block_array.extend(quote! { #tag_string });
                    }

                    let speed = match rule.speed {
                        Some(speed) => {
                            quote! { Some(#speed) }
                        }
                        None => quote! { None },
                    };
                    let correct_for_drops = match rule.correct_for_drops {
                        Some(correct_for_drops) => {
                            let correct_for_drops =
                                LitBool::new(correct_for_drops, Span::call_site());
                            quote! { Some(#correct_for_drops) }
                        }
                        None => quote! { None },
                    };
                    quote! {
                        ToolRule {
                            blocks: &[#(#block_array),*],
                            speed: #speed,
                            correct_for_drops: #correct_for_drops
                        }
                    }
                });
                let damage_per_block = match tool.damage_per_block {
                    Some(speed) => {
                        let speed = LitInt::new(&speed.to_string(), Span::call_site());
                        quote! { Some(#speed) }
                    }
                    None => quote! { None },
                };
                let default_mining_speed = match tool.default_mining_speed {
                    Some(speed) => {
                        let speed = LitFloat::new(&speed.to_string(), Span::call_site());
                        quote! { Some(#speed) }
                    }
                    None => quote! { None },
                };
                quote! { Some(ToolComponent { rules: &[#(#rules_code),*], damage_per_block: #damage_per_block, default_mining_speed: #default_mining_speed  }) }
            }
            None => quote! { None },
        };

        let food = match &self.food {
            Some(food) => {
                let nutrition = LitInt::new(&food.nutrition.to_string(), Span::call_site());
                let saturation =
                    LitFloat::new(&format!("{:.1}", food.saturation), Span::call_site());
                let can_always_eat = match food.can_always_eat {
                    Some(can) => {
                        let can = LitBool::new(can, Span::call_site());
                        quote! { Some(#can) }
                    }
                    None => quote! { None },
                };
                quote! { Some(FoodComponent {
                    nutrition: #nutrition,
                    saturation: #saturation,
                    can_always_eat: #can_always_eat,
                } ) }
            }
            None => quote! { None },
        };
        let equippable = match &self.equippable {
            Some(equippable) => {
                let slot = LitStr::new(&equippable.slot, Span::call_site());
                let equip_sound = equippable
                    .equip_sound
                    .as_ref()
                    .map(|s| {
                        let equip_sound = LitStr::new(s, Span::call_site());
                        quote! { Some(#equip_sound) }
                    })
                    .unwrap_or(quote! { None });
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
                            .get_values()
                            .iter()
                            .map(|reg| {
                                let reg = LitStr::new(&reg.serialize(), Span::call_site());
                                quote! { #reg }
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
                let equip_on_interact =
                    LitBool::new(equippable.equip_on_interact, Span::call_site());
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

                quote! { Some(EquippableComponent {
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
                }) }
            }
            None => quote! { None },
        };

        tokens.extend(quote! {
            ItemComponents {
                item_name: #item_name,
                max_stack_size: #max_stack_size,
                jukebox_playable: #jukebox_playable,
                damage: #damage,
                max_damage: #max_damage,
                attribute_modifiers: #attribute_modifiers,
                tool: #tool,
                food: #food,
                equippable: #equippable,
            }
        });
    }
}
#[derive(Deserialize, Clone, Debug)]
pub struct ToolComponent {
    rules: Vec<ToolRule>,
    default_mining_speed: Option<f32>,
    damage_per_block: Option<u32>,
}

#[derive(Deserialize, Copy, Clone, Debug)]
pub struct FoodComponent {
    nutrition: u8,
    saturation: f32,
    can_always_eat: Option<bool>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ToolRule {
    blocks: RegistryEntryList,
    speed: Option<f32>,
    correct_for_drops: Option<bool>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Modifier {
    pub r#type: String,
    pub id: String,
    pub amount: f64,
    pub operation: Operation,
    // TODO: Make this an enum
    pub slot: String,
}

fn _true() -> bool {
    true
}

#[allow(dead_code)]
#[derive(Deserialize, Clone, Debug)]
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

    let items: HashMap<String, Item> =
        serde_json::from_str(&fs::read_to_string("../assets/items.json").unwrap())
            .expect("Failed to parse items.json");

    let mut type_from_raw_id_arms = TokenStream::new();
    let mut type_from_name = TokenStream::new();

    let mut constants = TokenStream::new();

    for (name, item) in items {
        let const_ident = format_ident!("{}", name.to_shouty_snake_case());

        let components = &item.components;
        let components_tokens = components.to_token_stream();

        let id_lit = LitInt::new(&item.id.to_string(), proc_macro2::Span::call_site());

        constants.extend(quote! {
            pub const #const_ident: Item = Item {
                id: #id_lit,
                registry_key: #name,
                components: #components_tokens
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
        use std::hash::{Hash, Hasher};
        use pumpkin_util::text::TextComponent;
        use crate::tag::{Tagable, RegistryKey};

        #[derive(Clone, Debug)]
        pub struct Item {
            pub id: u16,
            pub registry_key: &'static str,
            pub components: ItemComponents,
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

        #[derive(Clone, Copy, Debug)]
        pub struct ItemComponents {
            pub item_name: &'static str,
            pub max_stack_size: u8,
            pub jukebox_playable: Option<&'static str>,
            pub damage: Option<u16>,
            pub max_damage: Option<u16>,
            pub attribute_modifiers: Option<&'static [Modifier]>,
            pub tool: Option<ToolComponent>,
            pub food: Option<FoodComponent>,
            pub equippable: Option<EquippableComponent>,
        }

        #[derive(Clone, Copy, Debug)]
        pub struct Modifier {
            pub r#type: &'static str,
            pub id: &'static str,
            pub amount: f64,
            pub operation: Operation,
            // TODO: Make this an enum
            pub slot: &'static str,
        }

        #[derive(Clone, Copy, Debug, PartialEq)]
        pub enum Operation {
            AddValue,
            AddMultipliedBase,
            AddMultipliedTotal,
        }

        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct ToolComponent {
            pub rules: &'static [ToolRule],
            pub default_mining_speed: Option<f32>,
            pub damage_per_block: Option<u32>,
        }

        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct ToolRule {
            pub blocks: &'static [&'static str],
            pub speed: Option<f32>,
            pub correct_for_drops: Option<bool>,
        }

        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct FoodComponent {
            pub nutrition: u8,
            pub saturation: f32,
            pub can_always_eat: Option<bool>,
        }

        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct EquippableComponent {
            pub slot: &'static str,
            pub equip_sound: Option<&'static str>,
            pub asset_id: Option<&'static str>,
            pub camera_overlay: Option<&'static str>,
            pub allowed_entities: Option<&'static [&'static str]>,
            pub dispensable: bool,
            pub swappable: bool,
            pub damage_on_hurt: bool,
            pub equip_on_interact: bool,
            pub can_be_sheared: bool,
            pub shearing_sound: Option<&'static str>,
        }

        impl Item {
            #constants

            pub fn translated_name(&self) -> TextComponent {
                TextComponent::text(self.components.item_name)
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

            #[doc = "Try to get default components from a resource location string."]
            pub fn default_components(name: &str) -> Option<ItemComponents> {
                let item = Self::from_registry_key(name)?;
                Some(item.components)
            }
        }

        impl Tagable for Item {
            #[inline]
            fn tag_key() -> RegistryKey {
                RegistryKey::Item
            }

            #[inline]
            fn registry_key(&self) -> &str {
                self.registry_key
            }
        }
    }
}
