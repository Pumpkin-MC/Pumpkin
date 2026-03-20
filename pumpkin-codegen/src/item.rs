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

/// Deserialized item entry from `items.json`.
#[derive(Deserialize)]
pub struct Item {
    /// Numeric protocol ID for this item.
    pub id: u16,
    /// All data components attached to this item.
    pub components: ItemComponents,
}

/// All deserialized data components for a single item as stored in `items.json`.
#[derive(Deserialize)]
pub struct ItemComponents {
    /// Display name translation component.
    #[serde(rename = "minecraft:item_name")]
    pub item_name: TextComponent,
    /// Maximum number of items per stack.
    #[serde(rename = "minecraft:max_stack_size")]
    pub max_stack_size: u8,
    /// Jukebox song key if this item is a music disc, otherwise `None`.
    #[serde(rename = "minecraft:jukebox_playable")]
    pub jukebox_playable: Option<String>,
    /// Current damage value of a damageable item, if any.
    #[serde(rename = "minecraft:damage")]
    pub damage: Option<u16>,
    /// Maximum durability of a damageable item, if any.
    #[serde(rename = "minecraft:max_damage")]
    pub max_damage: Option<u16>,
    /// Attribute modifiers applied when the item is held or worn, if any.
    #[serde(rename = "minecraft:attribute_modifiers")]
    pub attribute_modifiers: Option<Vec<Modifier>>,
    /// Tool component containing mining rules, if this item is a tool.
    #[serde(rename = "minecraft:tool")]
    pub tool: Option<ToolComponent>,
    /// Food component, present if this item is edible.
    #[serde(rename = "minecraft:food")]
    pub food: Option<FoodComponent>,
    /// Equippable component, present if this item can be worn in an armor slot.
    #[serde(rename = "minecraft:equippable")]
    pub equippable: Option<EquippableComponent>,
    /// Consumable component, present if this item has a custom use animation or duration.
    #[serde(rename = "minecraft:consumable")]
    pub consumable: Option<Consumable>,
    /// Present if this item can block attacks (e.g., shields).
    #[serde(rename = "minecraft:blocks_attacks")]
    pub blocks_attacks: Option<BlocksAttacks>,
    /// Present if this item grants death protection (e.g., totem of undying).
    #[serde(rename = "minecraft:death_protection")]
    pub death_protection: Option<DeathProtection>,
    /// Damage type resistance, if this item is immune to specific damage types.
    #[serde(rename = "minecraft:damage_resistant")]
    pub damage_resistant: Option<DamageResistantComponent>,
    #[serde(rename = "minecraft:weapon")]
    pub weapon: Option<WeaponComponent>,
}

impl ToTokens for ItemComponents {
    /// Emits a sequence of `(DataComponent, &impl DataComponentImpl)` tuple expressions for code generation.
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
        } = *self.item_name.clone().0.content
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
        }

        if let Some(md) = self.max_damage {
            let max_damage_lit = LitInt::new(&md.to_string(), Span::call_site());
            tokens.extend(quote! {
                (MaxDamage, &MaxDamageImpl {
                    max_damage: #max_damage_lit,
                }),
            });
        }

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
        }

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
                        let ident =
                            format_ident!("{}", str.replace([':', '/'], "_").to_uppercase());
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
                let speed = if let Some(speed) = rule.speed {
                    quote! { Some(#speed) }
                } else {
                    quote! { None }
                };
                let correct_for_drops = if let Some(correct_for_drops) = rule.correct_for_drops {
                    let correct_for_drops = LitBool::new(correct_for_drops, Span::call_site());
                    quote! { Some(#correct_for_drops) }
                } else {
                    quote! { None }
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
        }

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
        }

        if let Some(consumable) = &self.consumable {
            let consume_seconds = LitFloat::new(
                &format!("{:.1}", consumable.consume_seconds.unwrap_or(1.6)),
                Span::call_site(),
            );

            tokens.extend(quote! { (Consumable, &ConsumableImpl {
                consume_seconds: #consume_seconds,
            }), });
        }

        if self.blocks_attacks.is_some() {
            tokens.extend(quote! { (BlocksAttacks, &BlocksAttacksImpl), });
        }

        if self.death_protection.is_some() {
            tokens.extend(quote! { (DeathProtection, &DeathProtectionImpl), });
        }

        if let Some(weapon) = &self.weapon {
            let damage = LitInt::new(&weapon.item_damage_per_attack.to_string(), Span::call_site());
            tokens.extend(quote! { (Weapon, &WeaponImpl { item_damage_per_attack: #damage }), });
        }

        if let Some(damage_resistant) = &self.damage_resistant {
            let res_type_variant = match damage_resistant.types.as_str() {
                // Common canonical and shorthand forms mapped to enum variant names
                "#minecraft:always_hurts_ender_dragons"
                | "minecraft:always_hurts_ender_dragons"
                | "always_hurts_ender_dragons" => "AlwaysHurtsEnderDragons",
                "#minecraft:always_kills_armor_stands"
                | "minecraft:always_kills_armor_stands"
                | "always_kills_armor_stands" => "AlwaysKillsArmorStands",
                "#minecraft:always_most_significant_fall"
                | "minecraft:always_most_significant_fall"
                | "always_most_significant_fall" => "AlwaysMostSignificantFall",
                "#minecraft:always_triggers_silverfish"
                | "minecraft:always_triggers_silverfish"
                | "always_triggers_silverfish" => "AlwaysTriggersSilverfish",
                "#minecraft:avoids_guardian_thorns"
                | "minecraft:avoids_guardian_thorns"
                | "avoids_guardian_thorns" => "AvoidsGuardianThorns",
                "#minecraft:burns_armor_stands"
                | "minecraft:burns_armor_stands"
                | "burns_armor_stands" => "BurnsArmorStands",
                "#minecraft:burn_from_stepping"
                | "minecraft:burn_from_stepping"
                | "burn_from_stepping" => "BurnFromStepping",
                "#minecraft:bypasses_armor" | "minecraft:bypasses_armor" | "bypasses_armor" => {
                    "BypassesArmor"
                }
                "#minecraft:bypasses_cooldown"
                | "minecraft:bypasses_cooldown"
                | "bypasses_cooldown" => "BypassesCooldown",
                "#minecraft:bypasses_effects"
                | "minecraft:bypasses_effects"
                | "bypasses_effects" => "BypassesEffects",
                "#minecraft:bypasses_enchantments"
                | "minecraft:bypasses_enchantments"
                | "bypasses_enchantments" => "BypassesEnchantments",
                "#minecraft:bypasses_invulnerability"
                | "minecraft:bypasses_invulnerability"
                | "bypasses_invulnerability" => "BypassesInvulnerability",
                "#minecraft:bypasses_resistance"
                | "minecraft:bypasses_resistance"
                | "bypasses_resistance" => "BypassesResistance",
                "#minecraft:bypasses_shield" | "minecraft:bypasses_shield" | "bypasses_shield" => {
                    "BypassesShield"
                }
                "#minecraft:bypasses_wolf_armor"
                | "minecraft:bypasses_wolf_armor"
                | "bypasses_wolf_armor" => "BypassesWolfArmor",
                "#minecraft:can_break_armor_stand"
                | "minecraft:can_break_armor_stand"
                | "can_break_armor_stand" => "CanBreakArmorStands",
                "#minecraft:damages_helmet" | "minecraft:damages_helmet" | "damages_helmet" => {
                    "DamagesHelmet"
                }
                "#minecraft:ignites_armor_stands"
                | "minecraft:ignites_armor_stands"
                | "ignites_armor_stands" => "IgnitesArmorStands",
                "#minecraft:is_drowning" | "minecraft:is_drowning" | "is_drowning" => "Drowning",
                "#minecraft:is_explosion"
                | "minecraft:is_explosion"
                | "is_explosion"
                | "explosion" => "Explosion",
                "#minecraft:is_fall" | "minecraft:is_fall" | "is_fall" | "fall" => "Fall",
                "#minecraft:is_fire" | "minecraft:is_fire" | "is_fire" | "fire" | "in_fire"
                | "minecraft:in_fire" => "Fire",
                "#minecraft:is_freezing" | "minecraft:is_freezing" | "is_freezing" => "Freezing",
                "#minecraft:is_lightning" | "minecraft:is_lightning" | "is_lightning" => {
                    "Lightning"
                }
                "#minecraft:is_player_attack"
                | "minecraft:is_player_attack"
                | "is_player_attack" => "PlayerAttack",
                "#minecraft:is_projectile" | "minecraft:is_projectile" | "is_projectile" => {
                    "Projectile"
                }
                "#minecraft:mace_smash" | "minecraft:mace_smash" | "mace_smash" => "MaceSmash",
                "#minecraft:no_anger" | "minecraft:no_anger" | "no_anger" => "NoAnger",
                "#minecraft:no_impact" | "minecraft:no_impact" | "no_impact" => "NoImpact",
                "#minecraft:no_knockback" | "minecraft:no_knockback" | "no_knockback" => {
                    "NoKnockback"
                }
                "#minecraft:panic_causes" | "minecraft:panic_causes" | "panic_causes" => {
                    "PanicCauses"
                }
                "#minecraft:panic_environmental_causes"
                | "minecraft:panic_environmental_causes"
                | "panic_environmental_causes" => "PanicEnvironmentalCauses",
                "#minecraft:witch_resistant_to"
                | "minecraft:witch_resistant_to"
                | "witch_resistant_to" => "WitchResistantTo",
                "#minecraft:wither_immune_to"
                | "minecraft:wither_immune_to"
                | "wither_immune_to" => "WitherImmuneTo",
                _ => "Generic",
            };
            let res_type_ident = format_ident!("{}", res_type_variant);
            tokens.extend(quote! { (DamageResistant, &DamageResistantImpl {
                res_type: DamageResistantType::#res_type_ident,
            }), });
        }

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
                                        tag.replace([':', '/'], "_").to_uppercase()
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
        }
    }
}

/// Serde default helper returning `1f32`.
const fn return_1f32() -> f32 {
    1.
}

/// Serde default helper returning `true`.
const fn return_true() -> bool {
    true
}

/// Deserialized tool component containing mining rules and default speed.
const fn default_item_damage() -> u32 {
    1
}

#[derive(Deserialize)]
pub struct ToolComponent {
    /// Ordered list of mining rules applied per block or block tag.
    rules: Vec<ToolRule>,
    /// Default mining speed when no rule matches, defaults to `1.0`.
    #[serde(default = "return_1f32")]
    default_mining_speed: f32,
    /// Durability consumed per block broken, defaults to `1`.
    #[serde(default = "default_item_damage")]
    damage_per_block: u32,
    /// Whether the tool can destroy blocks in creative mode, defaults to `true`.
    #[serde(default = "return_true")]
    can_destroy_blocks_in_creative: bool,
}

/// Serde default helper returning `false`.
const fn return_false() -> bool {
    false
}

/// Deserialized food component describing nutrition and saturation values.
#[derive(Deserialize, Copy, Clone)]
pub struct FoodComponent {
    /// Hunger points restored when eaten.
    nutrition: u8,
    /// Saturation modifier applied on eating.
    saturation: f32,
    /// Whether the item can be eaten even at full hunger, defaults to `false`.
    #[serde(default = "return_false")]
    can_always_eat: bool,
}

/// A single tool mining rule that maps a block set to an optional speed override.
#[derive(Deserialize, Clone)]
pub struct ToolRule {
    /// The block or block-tag set this rule applies to.
    blocks: RegistryEntryList,
    /// Optional speed override when mining matching blocks.
    speed: Option<f32>,
    /// Whether the tool yields drops for matching blocks, if specified.
    correct_for_drops: Option<bool>,
}

/// A single attribute modifier applied when the item is held or worn.
#[derive(Deserialize, Clone)]
pub struct Modifier {
    /// Namespaced attribute key (e.g., `"minecraft:attack_damage"`).
    pub r#type: String,
    /// Unique identifier for this modifier instance.
    pub id: String,
    /// Numeric value added or multiplied depending on `operation`.
    pub amount: f64,
    /// How `amount` is combined with the base attribute value.
    pub operation: Operation,
    // TODO: Make this an enum
    /// Equipment slot in which this modifier is active.
    pub slot: AttributeModifierSlot,
}

/// Serde default helper returning `true`.
const fn _true() -> bool {
    true
}

/// Deserialized consumable component describing use duration.
#[derive(Deserialize, Clone)]
pub struct Consumable {
    /// Time in seconds to fully consume the item; defaults to `1.6` if absent.
    consume_seconds: Option<f32>, // TODO
}

/// Deserialized death-protection component (e.g., totem of undying); fields are unimplemented.
#[derive(Deserialize, Clone)]
pub struct DeathProtection {
    // TODO
}

/// Deserialized attack-blocking component (e.g., shield); fields are unimplemented.
#[derive(Deserialize, Clone)]
pub struct WeaponComponent {
    #[serde(default = "default_item_damage")]
    pub item_damage_per_attack: u32,
    // TODO: Add disable_blocking_for_seconds parsing when shield-disable mechanic is implemented.
    // This preserves round-trip fidelity for vanilla items and datapacks.
}

#[derive(Deserialize, Clone)]
pub struct BlocksAttacks {
    // TODO
}

/// Deserialized damage-resistance component indicating which damage types the item resists.
#[derive(Deserialize, Clone)]
pub struct DamageResistantComponent {
    /// Namespaced damage type tag the item is immune to.
    pub types: String,
}

/// Deserialized equippable component describing how an item is worn or equipped.
#[derive(Deserialize, Clone)]
pub struct EquippableComponent {
    /// Equipment slot the item occupies (e.g., `"head"`, `"chest"`).
    pub slot: String,
    /// Sound event played when the item is equipped; uses a generic fallback if absent.
    pub equip_sound: Option<String>,
    /// Texture asset identifier for the equipped model, if any.
    pub asset_id: Option<String>,
    /// Screen overlay texture shown while equipped, if any.
    pub camera_overlay: Option<String>,
    /// Entities that can wear this item; all entities allowed when absent.
    pub allowed_entities: Option<RegistryEntryList>,
    /// Whether the item can be dispensed into an equipment slot, defaults to `true`.
    #[serde(default = "_true")]
    pub dispensable: bool,
    /// Whether shift-clicking swaps the item into the equipment slot, defaults to `true`.
    #[serde(default = "_true")]
    pub swappable: bool,
    /// Whether the item takes damage when the wearer is hurt, defaults to `true`.
    #[serde(default = "_true")]
    pub damage_on_hurt: bool,
    /// Whether right-clicking an entity equips the item, defaults to `false`.
    #[serde(default)]
    pub equip_on_interact: bool,
    /// Whether shears can remove this item from an entity, defaults to `false`.
    #[serde(default)]
    pub can_be_sheared: bool,
    /// Sound event played when sheared off, if any.
    pub shearing_sound: Option<String>,
}

/// Arithmetic operation applied when combining an attribute modifier's amount with the base value.
#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[expect(clippy::enum_variant_names)]
pub enum Operation {
    /// Adds the modifier amount directly to the base value.
    AddValue,
    /// Adds `amount * base` to the running total.
    AddMultipliedBase,
    /// Multiplies the running total by `1 + amount`.
    AddMultipliedTotal,
}

/// Reads `items.json` and generates the complete item registry `TokenStream`.
pub fn build() -> TokenStream {
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
