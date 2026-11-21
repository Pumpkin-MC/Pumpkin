use crate::block::{Block, BlockAssets};
use crate::enchantments::AttributeModifierSlot;
use heck::{ToPascalCase, ToShoutySnakeCase};
use proc_macro2::{Span, TokenStream};
use pumpkin_nbt::{Nbt, compound::NbtCompound};
use pumpkin_util::registry::TagType;
use pumpkin_util::text::TextContent;
use pumpkin_util::{registry::RegistryEntryList, text::TextComponent};
use quote::{ToTokens, format_ident, quote};
use serde::Deserialize;
use serde_repr::Deserialize_repr;
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    io::Cursor,
};
use syn::{Ident, LitBool, LitByteStr, LitFloat, LitInt, LitStr};

/// Deserialized item entry from `items.json`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Item {
    /// Numeric protocol ID for this item.
    pub id: u16,
    /// Item left behind when this item is consumed, from `Item.Properties.craftRemainder`.
    pub crafting_remainder: Option<ItemStackTemplateComponent>,
    /// All data components attached to this item.
    pub components: ItemComponents,
}

/// All deserialized data components for a single item as stored in `items.json`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemComponents {
    /// Display name translation component.
    #[serde(rename = "minecraft:item_name")]
    pub item_name: TextComponent,
    /// Maximum number of items per stack.
    #[serde(rename = "minecraft:max_stack_size")]
    pub max_stack_size: u8,
    /// Use cooldown component.
    #[serde(rename = "minecraft:use_cooldown")]
    pub use_cooldown: Option<UseCooldownComponent>,
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
    #[serde(rename = "minecraft:enchantable")]
    pub enchantable: Option<EnchantableComponent>,
    #[serde(rename = "minecraft:attack_range")]
    pub attack_range: Option<AttackRangeComponent>,
    #[serde(rename = "minecraft:banner_patterns")]
    pub banner_patterns: Option<Vec<serde_json::Value>>,
    #[serde(rename = "minecraft:bees")]
    pub bees: Option<Vec<serde_json::Value>>,
    #[serde(rename = "minecraft:block_state")]
    pub block_state: Option<BTreeMap<String, String>>,
    #[serde(rename = "minecraft:block_transformer")]
    pub block_transformer: Option<String>,
    #[serde(rename = "minecraft:break_sound")]
    pub break_sound: Option<String>,
    #[serde(rename = "minecraft:brewing_fuel")]
    pub brewing_fuel: Option<BrewingFuelComponent>,
    #[serde(rename = "minecraft:bucket_entity_data")]
    pub bucket_entity_data: Option<BTreeMap<String, serde_json::Value>>,
    #[serde(rename = "minecraft:bundle_contents")]
    pub bundle_contents: Option<Vec<serde_json::Value>>,
    #[serde(rename = "minecraft:charged_projectiles")]
    pub charged_projectiles: Option<Vec<serde_json::Value>>,
    #[serde(rename = "minecraft:chicken/variant")]
    pub chicken_variant: Option<String>,
    #[serde(rename = "minecraft:container")]
    pub container: Option<Vec<serde_json::Value>>,
    #[serde(rename = "minecraft:compostable")]
    pub compostable: Option<CompostableComponent>,
    #[serde(rename = "minecraft:cooking_fuel")]
    pub cooking_fuel: Option<CookingFuelComponent>,
    #[serde(rename = "minecraft:cushion/color")]
    pub cushion_color: Option<String>,
    #[serde(rename = "minecraft:damage_type")]
    pub damage_type: Option<String>,
    #[serde(rename = "minecraft:debug_stick_state")]
    pub debug_stick_state: Option<BTreeMap<String, String>>,
    #[serde(rename = "minecraft:dye")]
    pub dye: Option<String>,
    #[serde(rename = "minecraft:enchantment_glint_override")]
    pub enchantment_glint_override: Option<bool>,
    #[serde(rename = "minecraft:enchantments")]
    pub enchantments: Option<BTreeMap<String, i32>>,
    #[serde(rename = "minecraft:entity_data")]
    pub entity_data: Option<EntityDataComponent>,
    #[serde(rename = "minecraft:fireworks")]
    pub fireworks: Option<FireworksComponent>,
    #[serde(rename = "minecraft:glider")]
    pub glider: Option<EmptyObject>,
    #[serde(rename = "minecraft:instrument")]
    pub instrument: Option<String>,
    #[serde(rename = "minecraft:interact_animation")]
    pub interact_animation: Option<SwingAnimationComponent>,
    #[serde(rename = "minecraft:item_model")]
    pub item_model: Option<String>,
    #[serde(rename = "minecraft:kinetic_weapon")]
    pub kinetic_weapon: Option<KineticWeaponComponent>,
    #[serde(rename = "minecraft:lore")]
    pub lore: Option<Vec<serde_json::Value>>,
    #[serde(rename = "minecraft:map_decorations")]
    pub map_decorations: Option<BTreeMap<String, serde_json::Value>>,
    #[serde(rename = "minecraft:minimum_attack_charge")]
    pub minimum_attack_charge: Option<f32>,
    #[serde(rename = "minecraft:mob_visibility")]
    pub mob_visibility: Option<MobVisibilityComponent>,
    #[serde(rename = "minecraft:ominous_bottle_amplifier")]
    pub ominous_bottle_amplifier: Option<i32>,
    #[serde(rename = "minecraft:piercing_weapon")]
    pub piercing_weapon: Option<PiercingWeaponComponent>,
    #[serde(rename = "minecraft:pot_decorations")]
    pub pot_decorations: Option<PotDecorationsComponent>,
    #[serde(rename = "minecraft:potion_contents")]
    pub potion_contents: Option<PotionContentsComponent>,
    #[serde(rename = "minecraft:potion_duration_scale")]
    pub potion_duration_scale: Option<f32>,
    #[serde(rename = "minecraft:provides_banner_patterns")]
    pub provides_banner_patterns: Option<String>,
    #[serde(rename = "minecraft:provides_pottery_pattern")]
    pub provides_pottery_pattern: Option<String>,
    #[serde(rename = "minecraft:provides_trim_material")]
    pub provides_trim_material: Option<String>,
    #[serde(rename = "minecraft:rarity")]
    pub rarity: Option<String>,
    #[serde(rename = "minecraft:recipes")]
    pub recipes: Option<Vec<String>>,
    #[serde(rename = "minecraft:repair_cost")]
    pub repair_cost: Option<i32>,
    #[serde(rename = "minecraft:repairable")]
    pub repairable: Option<RepairableComponent>,
    #[serde(rename = "minecraft:sign_text_back")]
    pub sign_text_back: Option<SignTextComponent>,
    #[serde(rename = "minecraft:sign_text_front")]
    pub sign_text_front: Option<SignTextComponent>,
    #[serde(rename = "minecraft:stored_enchantments")]
    pub stored_enchantments: Option<BTreeMap<String, i32>>,
    #[serde(rename = "minecraft:suspicious_stew_effects")]
    pub suspicious_stew_effects: Option<Vec<serde_json::Value>>,
    #[serde(
        rename = "minecraft:attack_animation",
        alias = "minecraft:swing_animation"
    )]
    pub swing_animation: Option<SwingAnimationComponent>,
    #[serde(rename = "minecraft:tooltip_display")]
    pub tooltip_display: Option<TooltipDisplayComponent>,
    #[serde(rename = "minecraft:use_effects")]
    pub use_effects: Option<UseEffectsComponent>,
    #[serde(rename = "minecraft:use_remainder")]
    pub use_remainder: Option<ItemStackTemplateComponent>,
    #[serde(rename = "minecraft:villager_food")]
    pub villager_food: Option<VillagerFoodComponent>,
    #[serde(rename = "minecraft:writable_book_content")]
    pub writable_book_content: Option<WritableBookComponent>,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct EmptyObject {}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct EntityDataComponent {
    pub id: String,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct FireworksComponent {
    #[serde(default = "default_flight_duration")]
    pub flight_duration: i32,
}

const fn default_flight_duration() -> i32 {
    1
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct PotionContentsComponent {
    pub potion: Option<String>,
    pub custom_color: Option<i32>,
    pub custom_name: Option<String>,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct SignTextComponent {
    pub messages: [String; 4],
    #[serde(default = "default_sign_color")]
    pub color: String,
    #[serde(default)]
    pub has_glowing_text: bool,
}

fn default_sign_color() -> String {
    "black".to_string()
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct TooltipDisplayComponent {
    #[serde(default)]
    pub hide_tooltip: bool,
    #[serde(default)]
    pub hidden_components: Vec<String>,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct WritableBookComponent {
    #[serde(default)]
    pub pages: Vec<String>,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct BrewingFuelComponent {
    pub uses: serde_json::Value,
    #[serde(default)]
    pub speed_multiplier: Option<serde_json::Value>,
}

fn default_swing_animation_type() -> String {
    "whack".to_string()
}

const fn default_swing_animation_duration() -> i32 {
    6
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct SwingAnimationComponent {
    #[serde(default = "default_swing_animation_type")]
    pub r#type: String,
    #[serde(default = "default_swing_animation_duration")]
    pub duration: i32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnchantableComponent {
    pub value: i32,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct CompostableComponent {
    pub layers: String,
}

const fn default_template_count() -> u8 {
    1
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ItemStackTemplateComponent {
    pub id: String,
    #[serde(default = "default_template_count")]
    pub count: u8,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct PotDecorationsComponent {
    pub back: Option<ItemStackTemplateComponent>,
    pub left: Option<ItemStackTemplateComponent>,
    pub right: Option<ItemStackTemplateComponent>,
    pub front: Option<ItemStackTemplateComponent>,
}

const fn default_use_effects_speed() -> f32 {
    0.2
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct UseEffectsComponent {
    #[serde(default)]
    pub can_sprint: bool,
    #[serde(default = "_true")]
    pub interact_vibrations: bool,
    #[serde(default = "default_use_effects_speed")]
    pub speed_multiplier: f32,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct VillagerFoodComponent {
    pub nutrition: i32,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct MobVisibilityComponent {
    pub targeting_entity_types: StringOrList,
    pub visibility: f32,
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

        if let Some(use_cooldown) = &self.use_cooldown {
            let seconds = LitFloat::new(&format!("{:.1}", use_cooldown.seconds), Span::call_site());
            let cooldown_group = if let Some(cd_group) = &use_cooldown.cooldown_group {
                quote! { Some(#cd_group) }
            } else {
                quote! { None }
            };

            tokens.extend(quote! {
                (UseCooldown, &UseCooldownImpl {
                    seconds: #seconds,
                    cooldown_group: #cooldown_group,
                }),
            });
        }

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
            bedrock_translate: _,
            with: _,
        } = *self.item_name.clone().0.content
        else {
            unreachable!()
        };
        let item_name = LitStr::new(&text, Span::call_site());
        tokens.extend(quote! {
            (ItemName, &ItemNameImpl {
                name: Cow::Borrowed(#item_name),
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
                let display = match modifier.display.as_ref().map(|d| d.r#type.as_str()) {
                    Some("hidden") => quote! { ModifierDisplay::Hidden },
                    _ => quote! { ModifierDisplay::Default },
                };

                quote! {
                    Modifier {
                        r#type: &Attributes::#r#type,
                        id: Cow::Borrowed(#id),
                        amount: #amount,
                        operation: Operation::#operation,
                        slot: #slot,
                        display: #display,
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
                            IDs(Cow::Borrowed(&[&Block::#ident]))
                        }
                    } else if let TagType::Tag(str) = t {
                        block_array = quote! {
                            Tag(Cow::Borrowed(#str))
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
                        IDs(Cow::Borrowed(&[#(#array),*]))
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
            let consume_particles = LitBool::new(
                consumable.has_consume_particles.unwrap_or(true),
                Span::call_site(),
            );

            let anim_str = consumable.animation.clone().unwrap_or("eat".to_string());
            let animation = format_ident!("{}", anim_str.to_pascal_case());

            let sound_id = consumable
                .sound
                .clone()
                .unwrap_or("minecraft:entity.generic.eat".to_string());
            let variant_name = format_ident!(
                "{}",
                sound_id
                    .strip_prefix("minecraft:")
                    .unwrap()
                    .to_pascal_case()
            );
            let effect_tokens =
                consume_effects_tokens(consumable.on_consume_effects.as_deref().unwrap_or(&[]));

            tokens.extend(quote! { (Consumable, &ConsumableImpl {
                consume_seconds: #consume_seconds,
                animation: ConsumeAnimation::#animation,
                sound_event: IdOr::Id(Sound::#variant_name),
                consume_particles: #consume_particles,
                effects: Cow::Borrowed(&[#effect_tokens])
            }), });
        }

        if let Some(blocks_attacks) = &self.blocks_attacks {
            let block_delay_seconds = float_literal(blocks_attacks.block_delay_seconds);
            let disable_cooldown_scale = float_literal(blocks_attacks.disable_cooldown_scale);
            let reductions_tokens: Vec<TokenStream> =
                if let Some(reds) = &blocks_attacks.damage_reductions {
                    reds.iter()
                        .map(|r| {
                            let angle = float_literal(r.horizontal_blocking_angle);
                            let dt = r.r#type.as_ref().map_or_else(
                                || quote! { None },
                                |t| {
                                    let set = damage_type_idset_tokens(t);
                                    quote! { Some(#set) }
                                },
                            );
                            let base = float_literal(r.base);
                            let factor = float_literal(r.factor);
                            quote! {
                                DamageReduction {
                                    horizontal_blocking_angle: #angle,
                                    damage_type: #dt,
                                    base: #base,
                                    factor: #factor,
                                }
                            }
                        })
                        .collect()
                } else {
                    vec![quote! { DamageReduction::DEFAULT }]
                };
            let item_damage_tokens = if let Some(idmg) = &blocks_attacks.item_damage {
                let threshold = float_literal(idmg.threshold);
                let base = float_literal(idmg.base);
                let factor = float_literal(idmg.factor);
                quote! {
                    ItemDamageFunction {
                        threshold: #threshold,
                        base: #base,
                        factor: #factor,
                    }
                }
            } else {
                quote! { ItemDamageFunction::DEFAULT }
            };
            let bypassed_by = blocks_attacks.bypassed_by.as_ref().map_or_else(
                || quote! { None },
                |b| {
                    let set = damage_type_idset_tokens(b);
                    quote! { Some(#set) }
                },
            );
            let block_sound = optional_sound_tokens(blocks_attacks.block_sound.as_deref());
            let disabled_sound = optional_sound_tokens(blocks_attacks.disabled_sound.as_deref());
            tokens.extend(quote! {
                (BlocksAttacks, &BlocksAttacksImpl {
                    block_delay_seconds: #block_delay_seconds,
                    disable_cooldown_scale: #disable_cooldown_scale,
                    damage_reductions: Cow::Borrowed(&[#(#reductions_tokens),*]),
                    item_damage: #item_damage_tokens,
                    bypassed_by: #bypassed_by,
                    block_sound: #block_sound,
                    disabled_sound: #disabled_sound,
                }),
            });
        }

        if let Some(death_protection) = &self.death_protection {
            let death_effect_tokens =
                consume_effects_tokens(death_protection.death_effects.as_deref().unwrap_or(&[]));
            tokens.extend(quote! {
                (DeathProtection, &DeathProtectionImpl {
                    death_effects: Cow::Borrowed(&[#death_effect_tokens]),
                }),
            });
        }

        if let Some(weapon) = &self.weapon {
            let damage = LitInt::new(
                &weapon.item_damage_per_attack.to_string(),
                Span::call_site(),
            );
            let disable_blocking_for_seconds = float_literal(weapon.disable_blocking_for_seconds);
            tokens.extend(quote! {
                (Weapon, &WeaponImpl {
                    item_damage_per_attack: #damage,
                    disable_blocking_for_seconds: #disable_blocking_for_seconds,
                }),
            });
        }

        if let Some(damage_resistant) = &self.damage_resistant {
            let types_tokens = damage_type_idset_tokens(&damage_resistant.types);
            tokens.extend(quote! {
                (DamageResistant, &DamageResistantImpl {
                    types: #types_tokens,
                }),
            });
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
                    let variant_name =
                        format_ident!("{}", s.strip_prefix("minecraft:").unwrap().to_pascal_case());
                    quote! { IdOr::Id(Sound::#variant_name) }
                })
                .unwrap_or(quote! { IdOr::Id(Sound::ItemArmorEquipGeneric) });
            let asset_id = equippable
                .asset_id
                .as_ref()
                .map(|s| {
                    let asset_id = LitStr::new(s, Span::call_site());
                    quote! { Some(Cow::Borrowed(#asset_id)) }
                })
                .unwrap_or(quote! { None });
            let camera_overlay = equippable
                .camera_overlay
                .as_ref()
                .map(|s| {
                    let camera_overlay = LitStr::new(s, Span::call_site());
                    quote! { Some(Cow::Borrowed(#camera_overlay)) }
                })
                .unwrap_or(quote! { None });
            let entities_option = equippable.allowed_entities.as_ref().map_or_else(
                || quote! { None },
                |entities| {
                    let allowed_entities = entity_idset_tokens(entities);
                    quote! { Some(#allowed_entities) }
                },
            );
            let dispensable = LitBool::new(equippable.dispensable, Span::call_site());
            let swappable = LitBool::new(equippable.swappable, Span::call_site());
            let damage_on_hurt = LitBool::new(equippable.damage_on_hurt, Span::call_site());
            let equip_on_interact = LitBool::new(equippable.equip_on_interact, Span::call_site());
            let can_be_sheared = LitBool::new(equippable.can_be_sheared, Span::call_site());
            let shearing_sound = equippable
                .shearing_sound
                .as_ref()
                .map(|s| {
                    let variant_name =
                        format_ident!("{}", s.strip_prefix("minecraft:").unwrap().to_pascal_case());
                    quote! { IdOr::Id(Sound::#variant_name) }
                })
                .unwrap_or(quote! { IdOr::Id(Sound::ItemShearsSnip) });

            tokens.extend(quote! { (Equippable, &EquippableImpl {
                slot: #slot,
                equip_sound: #equip_sound,
                asset_id: #asset_id,
                camera_overlay: #camera_overlay,
                allowed_entities: #entities_option,
                dispensable: #dispensable,
                swappable: #swappable,
                damage_on_hurt: #damage_on_hurt,
                equip_on_interact: #equip_on_interact,
                can_be_sheared: #can_be_sheared,
                shearing_sound: #shearing_sound
            }), });
        }

        if let Some(enchantable) = &self.enchantable {
            let value = LitInt::new(&enchantable.value.to_string(), Span::call_site());
            tokens.extend(quote! { (Enchantable, &EnchantableImpl { value: #value }), });
        }

        if let Some(attack_range) = &self.attack_range {
            let min_reach = float_literal(attack_range.min_reach);
            let max_reach = float_literal(attack_range.max_reach);
            let min_creative_reach = float_literal(attack_range.min_creative_reach);
            let max_creative_reach = float_literal(attack_range.max_creative_reach);
            let hitbox_margin = float_literal(attack_range.hitbox_margin);
            let mob_factor = float_literal(attack_range.mob_factor);
            tokens.extend(quote! {
                (AttackRange, &AttackRangeImpl {
                    min_reach: #min_reach,
                    max_reach: #max_reach,
                    min_creative_reach: #min_creative_reach,
                    max_creative_reach: #max_creative_reach,
                    hitbox_margin: #hitbox_margin,
                    mob_factor: #mob_factor,
                }),
            });
        }
        if self.banner_patterns.is_some() {
            tokens.extend(quote! { (BannerPatterns, &BannerPatternsImpl::EMPTY), });
        }
        if self.bees.is_some() {
            tokens.extend(quote! { (Bees, &BeesImpl), });
        }
        if let Some(block_state) = &self.block_state {
            let mut entries = TokenStream::new();
            for (k, v) in block_state {
                let k_lit = LitStr::new(k, Span::call_site());
                let v_lit = LitStr::new(v, Span::call_site());
                entries.extend(quote! {
                    (Cow::Borrowed(#k_lit), Cow::Borrowed(#v_lit)),
                });
            }
            tokens.extend(quote! {
                (BlockState, &BlockStateImpl {
                    properties: Cow::Borrowed(&[#entries])
                }),
            });
        }
        if let Some(transformer) = &self.block_transformer {
            let tr_lit = LitStr::new(transformer, Span::call_site());
            tokens.extend(quote! {
                (BlockTransformer, &BlockTransformerImpl {
                    transformer: Cow::Borrowed(#tr_lit),
                }),
            });
        }
        if let Some(break_sound) = &self.break_sound {
            let variant = format_ident!(
                "{}",
                break_sound
                    .strip_prefix("minecraft:")
                    .unwrap_or(break_sound)
                    .to_pascal_case()
            );
            tokens.extend(quote! {
                (BreakSound, &BreakSoundImpl {
                    sound: IdOr::Id(Sound::#variant),
                }),
            });
        }
        if self.bucket_entity_data.is_some() {
            tokens.extend(quote! { (BucketEntityData, &BucketEntityDataImpl), });
        }
        if self.bundle_contents.is_some() {
            tokens.extend(quote! { (BundleContents, &BundleContentsImpl::EMPTY), });
        }
        if self.charged_projectiles.is_some() {
            tokens.extend(quote! { (ChargedProjectiles, &ChargedProjectilesImpl { projectiles: Vec::new() }), });
        }
        if let Some(val_str) = &self.chicken_variant {
            let val_lit = LitStr::new(val_str, Span::call_site());
            tokens.extend(quote! { (ChickenVariant, &ChickenVariantImpl { value: Cow::Borrowed(#val_lit) }), });
        }
        if self.container.is_some() {
            tokens.extend(quote! { (Container, &ContainerImpl { items: Vec::new() }), });
        }
        if let Some(color) = &self.cushion_color {
            let color_ident = format_ident!("{}", color.to_pascal_case());
            tokens.extend(quote! {
                (CushionColor, &CushionColorImpl {
                    color: crate::dye_color::DyeColor::#color_ident,
                }),
            });
        }
        if let Some(damage_type) = &self.damage_type {
            let damage_type = format_ident!(
                "{}",
                damage_type
                    .strip_prefix("minecraft:")
                    .unwrap_or(damage_type)
                    .to_shouty_snake_case()
            );
            tokens.extend(quote! {
                (DamageType, &DamageTypeImpl {
                    damage_type: crate::damage::DamageType::#damage_type,
                }),
            });
        }
        if self.debug_stick_state.is_some() {
            tokens.extend(quote! { (DebugStickState, &DebugStickStateImpl), });
        }
        if let Some(dye) = &self.dye {
            let color_ident = format_ident!("{}", dye.to_pascal_case());
            tokens.extend(quote! {
                (Dye, &DyeImpl {
                    color: crate::dye_color::DyeColor::#color_ident,
                }),
            });
        }
        if let Some(brewing_fuel) = &self.brewing_fuel {
            let uses_tokens = int_provider_tokens(&brewing_fuel.uses);
            let speed_multiplier_tokens = float_provider_tokens(
                brewing_fuel.speed_multiplier.as_ref(),
                "minecraft:brewing/speed_default",
            );
            tokens.extend(quote! {
                (BrewingFuel, &BrewingFuelImpl {
                    uses: #uses_tokens,
                    speed_multiplier: #speed_multiplier_tokens,
                }),
            });
        }
        if let Some(compostable) = &self.compostable {
            let layers = compostable
                .layers
                .strip_prefix("minecraft:")
                .unwrap_or(&compostable.layers);
            let chance = match layers {
                "compostable/low" => 0.3,
                "compostable/low_medium" => 0.5,
                "compostable/medium" => 0.65,
                "compostable/medium_high" => 0.85,
                "compostable/always_add_one" => 1.0,
                _ => panic!("Unknown compostable layers: {}", compostable.layers),
            };
            let chance_lit = LitFloat::new(&format!("{chance:?}f32"), Span::call_site());
            tokens.extend(quote! {
                (Compostable, &CompostableImpl { chance: #chance_lit }),
            });
        }
        if let Some(fuel) = &self.cooking_fuel {
            let burn_time_tokens = int_provider_tokens(&fuel.burn_time);
            let speed_multiplier_tokens = float_provider_tokens(
                fuel.speed_multiplier.as_ref(),
                "minecraft:cooking/speed_default",
            );

            tokens.extend(quote! {
                (CookingFuel, &CookingFuelImpl {
                    burn_time: #burn_time_tokens,
                    speed_multiplier: #speed_multiplier_tokens,
                }),
            });
        }
        if let Some(glint) = self.enchantment_glint_override {
            let glint_lit = LitBool::new(glint, Span::call_site());
            tokens.extend(quote! {
                (EnchantmentGlintOverride, &EnchantmentGlintOverrideImpl { glint: #glint_lit }),
            });
        }
        if let Some(enchantments) = &self.enchantments {
            let entries: Vec<TokenStream> = enchantments
                .iter()
                .map(|(name, level)| {
                    let ench = format_ident!(
                        "{}",
                        name.strip_prefix("minecraft:")
                            .unwrap_or(name)
                            .to_shouty_snake_case()
                    );
                    let lvl = LitInt::new(&level.to_string(), Span::call_site());
                    quote! { (&crate::Enchantment::#ench, #lvl) }
                })
                .collect();
            tokens.extend(
                quote! { (Enchantments, &EnchantmentsImpl { enchantment: Cow::Borrowed(&[#(#entries),*]) }), },
            );
        }
        if let Some(entity_data) = &self.entity_data {
            let entity_ident = format_ident!(
                "{}",
                entity_data
                    .id
                    .strip_prefix("minecraft:")
                    .unwrap_or(&entity_data.id)
                    .to_shouty_snake_case()
            );
            tokens.extend(quote! {
                (EntityData, &EntityDataImpl {
                    entity_type: Some(&crate::entity_type::EntityType::#entity_ident),
                    nbt: None,
                }),
            });
        }
        if let Some(fireworks) = &self.fireworks {
            let flight_duration = fireworks.flight_duration;
            tokens.extend(quote! {
                (Fireworks, &FireworksImpl {
                    flight_duration: #flight_duration,
                    explosions: Vec::new(),
                }),
            });
        }
        if self.glider.is_some() {
            tokens.extend(quote! { (Glider, &GliderImpl), });
        }
        if let Some(instrument) = &self.instrument {
            let inst_lit = LitStr::new(instrument, Span::call_site());
            tokens.extend(quote! {
                (Instrument, &InstrumentImpl {
                    instrument: Cow::Borrowed(#inst_lit),
                }),
            });
        }
        if let Some(interact) = &self.interact_animation {
            let anim_type = match interact.r#type.as_str() {
                "whack" => quote! { SwingAnimationType::Whack },
                "stab" => quote! { SwingAnimationType::Stab },
                "none" => quote! { SwingAnimationType::None },
                _ => quote! { SwingAnimationType::Whack },
            };
            let duration = interact.duration;
            tokens.extend(quote! {
                (
                    InteractAnimation,
                    &InteractAnimationImpl {
                        animation_type: #anim_type,
                        duration: #duration,
                    },
                ),
            });
        }
        if let Some(model) = &self.item_model {
            let model_lit = LitStr::new(model, Span::call_site());
            tokens
                .extend(quote! { (ItemModel, &ItemModelImpl { id: Cow::Borrowed(#model_lit) }), });
        }
        if let Some(kinetic_weapon) = &self.kinetic_weapon {
            let contact_cooldown_ticks = LitInt::new(
                &kinetic_weapon.contact_cooldown_ticks.to_string(),
                Span::call_site(),
            );
            let delay_ticks =
                LitInt::new(&kinetic_weapon.delay_ticks.to_string(), Span::call_site());
            let dismount_conditions =
                kinetic_condition_tokens(kinetic_weapon.dismount_conditions.as_ref());
            let knockback_conditions =
                kinetic_condition_tokens(kinetic_weapon.knockback_conditions.as_ref());
            let damage_conditions =
                kinetic_condition_tokens(kinetic_weapon.damage_conditions.as_ref());
            let forward_movement = float_literal(kinetic_weapon.forward_movement);
            let damage_multiplier = float_literal(kinetic_weapon.damage_multiplier);
            let sound = optional_sound_tokens(kinetic_weapon.sound.as_deref());
            let hit_sound = optional_sound_tokens(kinetic_weapon.hit_sound.as_deref());
            tokens.extend(quote! {
                (KineticWeapon, &KineticWeaponImpl {
                    contact_cooldown_ticks: #contact_cooldown_ticks,
                    delay_ticks: #delay_ticks,
                    dismount_conditions: #dismount_conditions,
                    knockback_conditions: #knockback_conditions,
                    damage_conditions: #damage_conditions,
                    forward_movement: #forward_movement,
                    damage_multiplier: #damage_multiplier,
                    sound: #sound,
                    hit_sound: #hit_sound,
                }),
            });
        }
        if self.lore.is_some() {
            tokens.extend(quote! { (Lore, &LoreImpl { lines: Vec::new() }), });
        }
        if self.map_decorations.is_some() {
            tokens.extend(quote! { (MapDecorations, &MapDecorationsImpl), });
        }
        if let Some(charge) = self.minimum_attack_charge {
            let charge = float_literal(charge);
            tokens.extend(
                quote! { (MinimumAttackCharge, &MinimumAttackChargeImpl { charge: #charge }), },
            );
        }
        if let Some(mob_vis) = &self.mob_visibility {
            let targeting_tokens = entity_idset_tokens(&mob_vis.targeting_entity_types);
            let visibility = float_literal(mob_vis.visibility);
            tokens.extend(quote! {
                (MobVisibility, &MobVisibilityImpl {
                    targeting_entity_types: #targeting_tokens,
                    visibility: #visibility,
                }),
            });
        }
        if let Some(amp) = self.ominous_bottle_amplifier {
            let amp_lit = LitInt::new(&amp.to_string(), Span::call_site());
            tokens.extend(quote! { (OminousBottleAmplifier, &OminousBottleAmplifierImpl { amplifier: #amp_lit }), });
        }
        if let Some(piercing_weapon) = &self.piercing_weapon {
            let deals_knockback = LitBool::new(piercing_weapon.deals_knockback, Span::call_site());
            let dismounts = LitBool::new(piercing_weapon.dismounts, Span::call_site());
            let sound = optional_sound_tokens(piercing_weapon.sound.as_deref());
            let hit_sound = optional_sound_tokens(piercing_weapon.hit_sound.as_deref());
            tokens.extend(quote! {
                (PiercingWeapon, &PiercingWeaponImpl {
                    deals_knockback: #deals_knockback,
                    dismounts: #dismounts,
                    sound: #sound,
                    hit_sound: #hit_sound,
                }),
            });
        }
        if let Some(pot) = &self.pot_decorations {
            let side_tokens = |side: &Option<ItemStackTemplateComponent>| {
                side.as_ref().map_or_else(
                    || quote! { None },
                    |s| {
                        let name = s.id.strip_prefix("minecraft:").unwrap_or(&s.id);
                        let ident = format_ident!("{}", name.to_shouty_snake_case());
                        quote! { Some(crate::item_stack::ItemStack::static_new_java(1, &Self::#ident)) }
                    },
                )
            };
            let back = side_tokens(&pot.back);
            let left = side_tokens(&pot.left);
            let right = side_tokens(&pot.right);
            let front = side_tokens(&pot.front);
            tokens.extend(quote! {
                (PotDecorations, &PotDecorationsImpl {
                    back: #back,
                    left: #left,
                    right: #right,
                    front: #front,
                }),
            });
        }
        if let Some(pc) = &self.potion_contents {
            let potion_id = pc.potion.as_ref().map_or_else(
                || quote! { None },
                |p| {
                    let ident = format_ident!(
                        "{}",
                        p.strip_prefix("minecraft:")
                            .unwrap_or(p)
                            .to_shouty_snake_case()
                    );
                    quote! { Some(crate::potion::Potion::#ident.id as i32) }
                },
            );
            let custom_color = pc.custom_color.map_or_else(
                || quote! { None },
                |c| {
                    let lit = LitInt::new(&c.to_string(), Span::call_site());
                    quote! { Some(#lit) }
                },
            );
            tokens.extend(quote! {
                (PotionContents, &PotionContentsImpl {
                    potion_id: #potion_id,
                    custom_color: #custom_color,
                    custom_effects: Vec::new(),
                    custom_name: None,
                }),
            });
        }
        if let Some(scale) = self.potion_duration_scale {
            let scale_lit = LitFloat::new(&format!("{scale:?}f32"), Span::call_site());
            tokens.extend(
                quote! { (PotionDurationScale, &PotionDurationScaleImpl { scale: #scale_lit }), },
            );
        }
        if let Some(key) = &self.provides_banner_patterns {
            let key_lit = LitStr::new(key, Span::call_site());
            tokens.extend(quote! {
                (ProvidesBannerPatterns, &ProvidesBannerPatternsImpl {
                    key: Cow::Borrowed(#key_lit),
                }),
            });
        }
        if let Some(pattern) = &self.provides_pottery_pattern {
            let pat_lit = LitStr::new(pattern, Span::call_site());
            tokens.extend(quote! {
                (ProvidesPotteryPattern, &ProvidesPotteryPatternImpl {
                    pattern: Cow::Borrowed(#pat_lit),
                }),
            });
        }
        if let Some(material) = &self.provides_trim_material {
            let mat_lit = LitStr::new(material, Span::call_site());
            tokens.extend(quote! {
                (ProvidesTrimMaterial, &ProvidesTrimMaterialImpl {
                    material: Cow::Borrowed(#mat_lit),
                }),
            });
        }
        if let Some(rarity_str) = &self.rarity {
            let rarity_variant = match rarity_str.as_str() {
                "uncommon" => quote! { crate::data_component_impl::Rarity::Uncommon },
                "rare" => quote! { crate::data_component_impl::Rarity::Rare },
                "epic" => quote! { crate::data_component_impl::Rarity::Epic },
                _ => quote! { crate::data_component_impl::Rarity::Common },
            };
            tokens.extend(quote! { (Rarity, &RarityImpl { rarity: #rarity_variant }), });
        }
        if let Some(recipes) = &self.recipes {
            let recipe_tokens: Vec<TokenStream> = recipes
                .iter()
                .map(|r| {
                    let lit = LitStr::new(r, Span::call_site());
                    quote! { Cow::Borrowed(#lit) }
                })
                .collect();
            tokens.extend(quote! {
                (Recipes, &RecipesImpl {
                    recipes: Cow::Borrowed(&[#(#recipe_tokens),*]),
                }),
            });
        }
        if let Some(cost) = self.repair_cost {
            tokens.extend(quote! { (RepairCost, &RepairCostImpl { cost: #cost }), });
        }
        if let Some(repairable) = &self.repairable {
            let mut items_tokens = TokenStream::new();
            match &repairable.items {
                StringOrList::String(str) => {
                    if let Some(formatted) = str.strip_prefix('#') {
                        items_tokens.extend(quote! {
                            IDSet::Tag(Cow::Borrowed(#formatted))
                        });
                    } else {
                        let item_name = str.strip_prefix("minecraft:").unwrap_or(str);
                        let ident = format_ident!("{}", item_name.to_shouty_snake_case());
                        items_tokens.extend(quote! {
                            IDSet::IDs(Cow::Borrowed(&[&Self::#ident]))
                        });
                    }
                }
                StringOrList::List(items) => {
                    let mut ids = TokenStream::new();
                    for x in items {
                        let item_name = x.strip_prefix("minecraft:").unwrap_or(x);
                        let ident = format_ident!("{}", item_name.to_shouty_snake_case());
                        ids.extend(quote! { &Self::#ident, });
                    }
                    items_tokens.extend(quote! {
                        IDSet::IDs(Cow::Borrowed(&[#ids]))
                    });
                }
            }
            tokens.extend(quote! { (Repairable, &RepairableImpl { items: #items_tokens }), });
        }
        if let Some(sign_back) = &self.sign_text_back {
            let sign_tokens = sign_text_tokens(sign_back, quote! { SignTextBackImpl });
            tokens.extend(quote! { (SignTextBack, &#sign_tokens), });
        }
        if let Some(sign_front) = &self.sign_text_front {
            let sign_tokens = sign_text_tokens(sign_front, quote! { SignTextFrontImpl });
            tokens.extend(quote! { (SignTextFront, &#sign_tokens), });
        }
        if let Some(stored_enchantments) = &self.stored_enchantments {
            let entries: Vec<TokenStream> = stored_enchantments
                .iter()
                .map(|(name, level)| {
                    let ench = format_ident!(
                        "{}",
                        name.strip_prefix("minecraft:")
                            .unwrap_or(name)
                            .to_shouty_snake_case()
                    );
                    let lvl = LitInt::new(&level.to_string(), Span::call_site());
                    quote! { (&crate::Enchantment::#ench, #lvl) }
                })
                .collect();
            tokens.extend(quote! { (StoredEnchantments, &StoredEnchantmentsImpl { enchantment: Cow::Borrowed(&[#(#entries),*]) }), });
        }
        if self.suspicious_stew_effects.is_some() {
            tokens.extend(quote! { (SuspiciousStewEffects, &SuspiciousStewEffectsImpl::EMPTY), });
        }
        if let Some(swing) = &self.swing_animation {
            let anim_type = match swing.r#type.as_str() {
                "whack" => quote! { SwingAnimationType::Whack },
                "stab" => quote! { SwingAnimationType::Stab },
                "none" => quote! { SwingAnimationType::None },
                _ => quote! { SwingAnimationType::Whack },
            };
            let duration = swing.duration;
            tokens.extend(quote! {
                (
                    AttackAnimation,
                    &SwingAnimationImpl {
                        animation_type: #anim_type,
                        duration: #duration,
                    },
                ),
            });
        }
        if let Some(td) = &self.tooltip_display {
            if !td.hide_tooltip && td.hidden_components.is_empty() {
                tokens.extend(quote! { (TooltipDisplay, &TooltipDisplayImpl::DEFAULT), });
            } else {
                let hide_tooltip = LitBool::new(td.hide_tooltip, Span::call_site());
                let hidden_comps: Vec<TokenStream> = td
                    .hidden_components
                    .iter()
                    .map(|name| {
                        let ident = format_ident!(
                            "{}",
                            name.strip_prefix("minecraft:")
                                .unwrap_or(name)
                                .replace('/', "_")
                                .to_pascal_case()
                        );
                        quote! { DataComponent::#ident }
                    })
                    .collect();
                tokens.extend(quote! {
                    (TooltipDisplay, &TooltipDisplayImpl {
                        hide_tooltip: #hide_tooltip,
                        hidden_components: Cow::Borrowed(&[#(#hidden_comps),*]),
                    }),
                });
            }
        }
        if let Some(use_effects) = &self.use_effects {
            let can_sprint = LitBool::new(use_effects.can_sprint, Span::call_site());
            let interact_vibrations =
                LitBool::new(use_effects.interact_vibrations, Span::call_site());
            let speed_multiplier = float_literal(use_effects.speed_multiplier);
            tokens.extend(quote! {
                (UseEffects, &UseEffectsImpl {
                    can_sprint: #can_sprint,
                    interact_vibrations: #interact_vibrations,
                    speed_multiplier: #speed_multiplier,
                }),
            });
        }
        if let Some(remainder) = &self.use_remainder {
            let name = remainder
                .id
                .strip_prefix("minecraft:")
                .unwrap_or(&remainder.id);
            let ident = format_ident!("{}", name.to_shouty_snake_case());
            let count = LitInt::new(&remainder.count.to_string(), Span::call_site());
            tokens.extend(quote! {
                (UseRemainder, &UseRemainderImpl {
                    remainder: Some(crate::item_stack::ItemStack::static_new_java(#count, &Self::#ident)),
                }),
            });
        }
        if let Some(villager_food) = &self.villager_food {
            let nutrition = LitInt::new(&villager_food.nutrition.to_string(), Span::call_site());
            tokens.extend(quote! {
                (VillagerFood, &VillagerFoodImpl {
                    nutrition: #nutrition,
                }),
            });
        }

        if self.writable_book_content.is_some() {
            tokens.extend(
                quote! { (WritableBookContent, &WritableBookContentImpl { pages: Vec::new() }), },
            );
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
#[serde(deny_unknown_fields)]
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UseCooldownComponent {
    seconds: f32,
    cooldown_group: Option<String>,
}

/// Serde default helper returning `false`.
const fn return_false() -> bool {
    false
}

/// Deserialized food component describing nutrition and saturation values.
#[derive(Deserialize, Copy, Clone)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct ToolRule {
    /// The block or block-tag set this rule applies to.
    blocks: RegistryEntryList,
    /// Optional speed override when mining matching blocks.
    speed: Option<f32>,
    /// Whether the tool yields drops for matching blocks, if specified.
    correct_for_drops: Option<bool>,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ModifierDisplayComponent {
    pub r#type: String,
    pub value: Option<serde_json::Value>,
}

fn default_modifier_slot() -> AttributeModifierSlot {
    AttributeModifierSlot::Any
}

/// A single attribute modifier applied when the item is held or worn.
#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Modifier {
    /// Namespaced attribute key (e.g., `"minecraft:attack_damage"`).
    pub r#type: String,
    /// Unique identifier for this modifier instance.
    pub id: String,
    /// Numeric value added or multiplied depending on `operation`.
    pub amount: f64,
    /// How `amount` is combined with the base attribute value.
    pub operation: Operation,
    /// Equipment slot in which this modifier is active.
    #[serde(default = "default_modifier_slot")]
    pub slot: AttributeModifierSlot,
    pub display: Option<ModifierDisplayComponent>,
}

/// Serde default helper returning `true`.
const fn _true() -> bool {
    true
}

/// Deserialized consumable component describing use duration.
#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Consumable {
    consume_seconds: Option<f32>,
    has_consume_particles: Option<bool>,
    animation: Option<String>,
    sound: Option<String>,
    on_consume_effects: Option<Vec<ConsumeEffect>>,
}
#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ConsumeEffect {
    r#type: String,
    probability: Option<f32>,
    diameter: Option<f32>,
    sound: Option<String>,
    effects: Option<StringOrStatusEffects>,
}
#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct StatusEffectInstance {
    pub id: String,
    pub amplifier: Option<i32>,
    pub duration: Option<i32>,
    pub ambient: Option<bool>,
    pub show_particles: Option<bool>,
    pub show_icon: Option<bool>,
}
#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum StringOrStatusEffects {
    String(String),
    List(Vec<String>),
    Effects(Vec<StatusEffectInstance>),
}

/// Deserialized death-protection component (e.g., totem of undying).
#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct DeathProtection {
    pub death_effects: Option<Vec<ConsumeEffect>>,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct WeaponComponent {
    #[serde(default = "default_item_damage")]
    pub item_damage_per_attack: u32,
    #[serde(default)]
    pub disable_blocking_for_seconds: f32,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct AttackRangeComponent {
    #[serde(default)]
    pub min_reach: f32,
    #[serde(default = "default_max_reach")]
    pub max_reach: f32,
    #[serde(default)]
    pub min_creative_reach: f32,
    #[serde(default = "default_max_creative_reach")]
    pub max_creative_reach: f32,
    #[serde(default = "default_hitbox_margin")]
    pub hitbox_margin: f32,
    #[serde(default = "return_1f32")]
    pub mob_factor: f32,
}

const fn default_max_reach() -> f32 {
    3.0
}

const fn default_max_creative_reach() -> f32 {
    5.0
}

const fn default_hitbox_margin() -> f32 {
    0.3
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct KineticConditionComponent {
    pub max_duration_ticks: i32,
    #[serde(default)]
    pub min_speed: f32,
    #[serde(default)]
    pub min_relative_speed: f32,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct KineticWeaponComponent {
    #[serde(default = "default_contact_cooldown_ticks")]
    pub contact_cooldown_ticks: i32,
    #[serde(default)]
    pub delay_ticks: i32,
    pub dismount_conditions: Option<KineticConditionComponent>,
    pub knockback_conditions: Option<KineticConditionComponent>,
    pub damage_conditions: Option<KineticConditionComponent>,
    #[serde(default)]
    pub forward_movement: f32,
    #[serde(default = "return_1f32")]
    pub damage_multiplier: f32,
    pub sound: Option<String>,
    pub hit_sound: Option<String>,
}

const fn default_contact_cooldown_ticks() -> i32 {
    10
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct PiercingWeaponComponent {
    #[serde(default = "_true")]
    pub deals_knockback: bool,
    #[serde(default)]
    pub dismounts: bool,
    pub sound: Option<String>,
    pub hit_sound: Option<String>,
}

fn float_literal(value: f32) -> LitFloat {
    LitFloat::new(&format!("{value:?}f32"), Span::call_site())
}

fn optional_sound_tokens(sound: Option<&str>) -> TokenStream {
    sound.map_or_else(
        || quote! { None },
        |sound| {
            let variant = format_ident!(
                "{}",
                sound
                    .strip_prefix("minecraft:")
                    .unwrap_or(sound)
                    .to_pascal_case()
            );
            quote! { Some(IdOr::Id(Sound::#variant)) }
        },
    )
}

fn kinetic_condition_tokens(condition: Option<&KineticConditionComponent>) -> TokenStream {
    condition.map_or_else(
        || quote! { None },
        |condition| {
            let max_duration_ticks =
                LitInt::new(&condition.max_duration_ticks.to_string(), Span::call_site());
            let min_speed = float_literal(condition.min_speed);
            let min_relative_speed = float_literal(condition.min_relative_speed);
            quote! {
                Some(KineticConditionImpl {
                    max_duration_ticks: #max_duration_ticks,
                    min_speed: #min_speed,
                    min_relative_speed: #min_relative_speed,
                })
            }
        },
    )
}

fn consume_effects_tokens(effects: &[ConsumeEffect]) -> TokenStream {
    let mut effect_tokens = TokenStream::new();
    for effect in effects {
        match effect.r#type.as_str() {
            "minecraft:clear_all_effects" => {
                effect_tokens.extend(quote! { ConsumeEffect::ClearAllEffects, });
            }
            "minecraft:teleport_randomly" => {
                let diameter = float_literal(effect.diameter.unwrap_or(16.0));
                effect_tokens.extend(quote! { ConsumeEffect::TeleportRandomly(#diameter), });
            }
            "minecraft:play_sound" => {
                let s = effect.sound.as_deref().expect("play_sound missing sound");
                let sound = format_ident!(
                    "{}",
                    s.strip_prefix("minecraft:").unwrap_or(s).to_pascal_case()
                );
                effect_tokens.extend(quote! { ConsumeEffect::PlaySound(IdOr::Id(Sound::#sound)), });
            }
            "minecraft:apply_effects" => {
                let probability = float_literal(effect.probability.unwrap_or(1.0));
                if let Some(StringOrStatusEffects::Effects(status_effect_instances)) =
                    &effect.effects
                {
                    let mut status_tokens = TokenStream::new();
                    for status in status_effect_instances {
                        let effect_id = LitStr::new(&status.id, Span::call_site());
                        let amplifier = status.amplifier.unwrap_or(0);
                        let duration = status.duration.unwrap_or(1);
                        let ambient = status.ambient.unwrap_or(false);
                        let show_particles = status.show_particles.unwrap_or(true);
                        let show_icon = status.show_icon.unwrap_or(true);
                        status_tokens.extend(quote! {
                            StatusEffectInstance {
                                effect_id: Cow::Borrowed(#effect_id),
                                amplifier: #amplifier,
                                duration: #duration,
                                ambient: #ambient,
                                show_particles: #show_particles,
                                show_icon: #show_icon
                            },
                        });
                    }
                    effect_tokens.extend(quote! {
                        ConsumeEffect::ApplyEffects((Cow::Borrowed(&[#status_tokens]), #probability)),
                    });
                }
            }
            "minecraft:remove_effects" => match effect.effects.as_ref().expect("remove_effects") {
                StringOrStatusEffects::String(id) => {
                    if let Some(tag) = id.strip_prefix('#') {
                        let lit = LitStr::new(tag, Span::call_site());
                        effect_tokens.extend(quote! {
                            ConsumeEffect::RemoveEffects(IDSet::Tag(Cow::Borrowed(#lit))),
                        });
                    } else {
                        let effect_id = format_ident!(
                            "{}",
                            id.strip_prefix("minecraft:")
                                .unwrap_or(id)
                                .to_shouty_snake_case()
                        );
                        effect_tokens.extend(quote! {
                            ConsumeEffect::RemoveEffects(IDSet::IDs(Cow::Borrowed(&[&StatusEffect::#effect_id]))),
                        });
                    }
                }
                StringOrStatusEffects::List(ids) => {
                    let idents: Vec<TokenStream> = ids
                        .iter()
                        .map(|id| {
                            let effect_id = format_ident!(
                                "{}",
                                id.strip_prefix("minecraft:")
                                    .unwrap_or(id)
                                    .to_shouty_snake_case()
                            );
                            quote! { &StatusEffect::#effect_id }
                        })
                        .collect();
                    effect_tokens.extend(quote! {
                        ConsumeEffect::RemoveEffects(IDSet::IDs(Cow::Borrowed(&[#(#idents),*]))),
                    });
                }
                StringOrStatusEffects::Effects(_) => {
                    panic!("Invalid remove_effects payload in items.json")
                }
            },
            other => panic!("Unknown ConsumeEffect type in items.json: {other}"),
        }
    }
    effect_tokens
}

fn int_provider_tokens(val: &serde_json::Value) -> TokenStream {
    match val {
        serde_json::Value::Number(n) => {
            let v = n.as_i64().unwrap_or(0) as i32;
            let lit = LitInt::new(&v.to_string(), Span::call_site());
            quote! { crate::data_component_impl::IntProvider::Inline(#lit) }
        }
        serde_json::Value::String(s) => {
            let lit = LitStr::new(s, Span::call_site());
            quote! { crate::data_component_impl::IntProvider::Id(std::borrow::Cow::Borrowed(#lit)) }
        }
        _ => panic!("Unexpected IntProvider value in items.json: {val}"),
    }
}

fn float_provider_tokens(val: Option<&serde_json::Value>, default_id: &str) -> TokenStream {
    match val {
        Some(serde_json::Value::Number(n)) => {
            let v = n.as_f64().unwrap_or(1.0) as f32;
            let lit = float_literal(v);
            quote! { crate::data_component_impl::FloatProvider::Inline(#lit) }
        }
        Some(serde_json::Value::String(s)) => {
            let lit = LitStr::new(s, Span::call_site());
            quote! { crate::data_component_impl::FloatProvider::Id(std::borrow::Cow::Borrowed(#lit)) }
        }
        None => {
            let lit = LitStr::new(default_id, Span::call_site());
            quote! { crate::data_component_impl::FloatProvider::Id(std::borrow::Cow::Borrowed(#lit)) }
        }
        Some(other) => panic!("Unexpected FloatProvider value in items.json: {other}"),
    }
}

fn sign_text_tokens(sign: &SignTextComponent, struct_tokens: TokenStream) -> TokenStream {
    let m0 = LitStr::new(&sign.messages[0], Span::call_site());
    let m1 = LitStr::new(&sign.messages[1], Span::call_site());
    let m2 = LitStr::new(&sign.messages[2], Span::call_site());
    let m3 = LitStr::new(&sign.messages[3], Span::call_site());
    let color = format_ident!("{}", sign.color.to_pascal_case());
    let glowing = LitBool::new(sign.has_glowing_text, Span::call_site());
    quote! {
        #struct_tokens {
            messages: [
                Cow::Borrowed(#m0),
                Cow::Borrowed(#m1),
                Cow::Borrowed(#m2),
                Cow::Borrowed(#m3),
            ],
            filtered_messages: None,
            color: crate::dye_color::DyeColor::#color,
            has_glowing_text: #glowing,
        }
    }
}

fn damage_type_idset_tokens(types: &StringOrList) -> TokenStream {
    let dt_ident = |name: &str| {
        let ident = format_ident!(
            "{}",
            name.strip_prefix("minecraft:")
                .unwrap_or(name)
                .to_shouty_snake_case()
        );
        quote! { &crate::damage::DamageType::#ident, }
    };
    match types {
        StringOrList::String(str) => {
            if let Some(tag) = str.strip_prefix('#') {
                let lit = LitStr::new(tag, Span::call_site());
                quote! { IDSet::Tag(Cow::Borrowed(#lit)) }
            } else {
                let ids = dt_ident(str);
                quote! { IDSet::IDs(Cow::Borrowed(&[#ids])) }
            }
        }
        StringOrList::List(items) => {
            let ids: TokenStream = items.iter().map(|name| dt_ident(name)).collect();
            quote! { IDSet::IDs(Cow::Borrowed(&[#ids])) }
        }
    }
}

const fn default_blocking_angle() -> f32 {
    90.0
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct DamageReductionComponent {
    #[serde(default = "default_blocking_angle")]
    pub horizontal_blocking_angle: f32,
    pub r#type: Option<StringOrList>,
    pub base: f32,
    pub factor: f32,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ItemDamageFunctionComponent {
    pub threshold: f32,
    pub base: f32,
    pub factor: f32,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct BlocksAttacks {
    #[serde(default)]
    pub block_delay_seconds: f32,
    #[serde(default = "return_1f32")]
    pub disable_cooldown_scale: f32,
    pub damage_reductions: Option<Vec<DamageReductionComponent>>,
    pub item_damage: Option<ItemDamageFunctionComponent>,
    pub bypassed_by: Option<StringOrList>,
    pub block_sound: Option<String>,
    pub disabled_sound: Option<String>,
}

/// Deserialized damage-resistance component indicating which damage types the item resists.
#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct DamageResistantComponent {
    /// Namespaced damage type tag or list the item is immune to.
    pub types: StringOrList,
}
#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum StringOrList {
    String(String),
    List(Vec<String>),
}

/// An entity type `IDSet`: one type, a list, or a `#tag`; `minecraft:` is optional.
fn entity_idset_tokens(entities: &StringOrList) -> TokenStream {
    let entity = |name: &str| {
        let ident = format_ident!(
            "{}",
            name.strip_prefix("minecraft:")
                .unwrap_or(name)
                .to_uppercase()
        );
        quote! { &crate::entity_type::EntityType::#ident, }
    };
    match entities {
        StringOrList::String(str) => {
            if let Some(tag) = str.strip_prefix('#') {
                let tag = tag.strip_prefix("minecraft:").unwrap_or(tag);
                quote! { IDSet::Tag(Cow::Borrowed(#tag)) }
            } else {
                let ids = entity(str);
                quote! { IDSet::IDs(Cow::Borrowed(&[#ids])) }
            }
        }
        StringOrList::List(items) => {
            let ids: TokenStream = items.iter().map(|name| entity(name)).collect();
            quote! { IDSet::IDs(Cow::Borrowed(&[#ids])) }
        }
    }
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct CookingFuelComponent {
    pub burn_time: serde_json::Value,
    #[serde(default)]
    pub speed_multiplier: Option<serde_json::Value>,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RepairableComponent {
    pub items: StringOrList,
}

/// Deserialized equippable component describing how an item is worn or equipped.
#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct EquippableComponent {
    /// Equipment slot the item occupies (e.g., `"head"`, `"chest"`).
    pub slot: String,
    /// Sound event played when the item is equipped; uses a generic fallback if absent.
    pub equip_sound: Option<String>,
    /// Texture asset identifier for the equipped model, if any.
    pub asset_id: Option<String>,
    /// Screen overlay texture shown while equipped, if any.
    pub camera_overlay: Option<String>,
    pub allowed_entities: Option<StringOrList>,
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

#[derive(Deserialize_repr, Debug)]
#[repr(i32)]
enum BedrockItemVersion {
    Legacy = 0,
    DataDriven = 1,
    None = 2,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct BedrockRuntimeItemState {
    name: String,
    id: i16,
    version: BedrockItemVersion,
    component_based: bool,
}

#[derive(Debug)]
enum BedrockToolType {
    Sword,
    Shovel,
    Pickaxe,
    Axe,
    Shears,
    Hoe,
}

#[derive(Debug)]
enum BedrockArmorType {
    Helmet,
    Leggings,
    Chestplate,
    Boots,
}

#[derive(Debug)]
struct BedrockRemapEntry {
    identifier: String,
    data: u32,
    block_runtime_id_range: Option<(i32, i32)>,
    tool_type: Option<BedrockToolType>,
    armor_type: Option<BedrockArmorType>,
    protection_value: Option<i32>,
    is_entity_placer: bool,
    is_edible: bool,
}

#[derive(Debug)]
pub struct BedrockItem {
    id: i16,
    registry_key: String,
    version: BedrockItemVersion,
    component_based: bool,
    components: Nbt,
}

/// Reads `items.json` and generates the complete item registry `TokenStream`.
pub fn build() -> TokenStream {
    let blocks_assets: BlockAssets =
        serde_json::from_str(&fs::read_to_string("../../assets/blocks.json").unwrap())
            .expect("Failed to parse blocks.json");

    let items: BTreeMap<String, Item> =
        serde_json::from_str(&fs::read_to_string("../../assets/items.json").unwrap())
            .expect("Failed to parse items.json");

    let be_item_components: BTreeMap<String, Option<NbtCompound>> = {
        let data = fs::read("../../assets/bedrock/item_components.nbt").unwrap();
        let mut cursor = Cursor::new(data);
        let nbt = pumpkin_nbt::nbt_compress::read_gzip_compound_tag(&mut cursor)
            .expect("Failed to parse bedrock/item_components.nbt");
        nbt.child_tags
            .into_iter()
            .map(|(k, v)| {
                let comp = match v {
                    pumpkin_nbt::tag::NbtTag::Compound(c) => Some(c),
                    _ => None,
                };
                (k.into_string(), comp)
            })
            .collect()
    };

    let be_runtime_item_states: Vec<BedrockRuntimeItemState> = serde_json::from_str(
        &fs::read_to_string("../../assets/bedrock/runtime_item_states.json").unwrap(),
    )
    .expect("Failed to parse bedrock/runtime_item_states.json");

    let be_item_data_overrides: BTreeMap<String, u32> = serde_json::from_str(
        &fs::read_to_string("../../assets/bedrock/item_data_overrides.json").unwrap(),
    )
    .expect("Failed to parse bedrock/item_data_overrides.json");

    let be_valid_item_identifiers = be_runtime_item_states
        .iter()
        .map(|item| &item.name)
        .collect::<HashSet<_>>();

    let mut be_item_remaps = indexmap::IndexMap::new();

    let mut item_to_block: BTreeMap<u16, &Block> = BTreeMap::new();
    for block in &blocks_assets.blocks {
        if block.item_id != 0 {
            item_to_block.entry(block.item_id).or_insert(block);
        }
    }

    for (name, item) in &items {
        // THIS COERCES CODEGEN FOR JAVA 26.3
        // This is for coercing the codegen into working for java 26.3 temporarily.
        /// Remove `let mut be_identifier =` and replace with `let be_identifier =` to undo this.
        let mut be_identifier = "minecraft:".to_owned()
            + &match &**name {
                "bricks" => "brick_block".into(),
                "cobblestone_stairs" => "stone_stairs".into(),
                "cobweb" => "web".into(),
                "dead_bush" => "deadbush".into(),
                "debug_stick" => "stick".into(),
                "dirt_path" => "grass_path".into(),
                "end_stone_bricks" => "end_bricks".into(),
                "end_stone_brick_stairs" => "end_brick_stairs".into(),
                "flowering_azalea_leaves" => "azalea_leaves_flowered".into(),
                "frogspawn" => "frog_spawn".into(),
                "furnace_minecart" => "hopper_minecart".into(),
                "glow_item_frame" => "glow_frame".into(),
                "item_frame" => "frame".into(),
                "jack_o_lantern" => "lit_pumpkin".into(),
                "light" => "light_block".into(),
                "light_gray_glazed_terracotta" => "silver_glazed_terracotta".into(),
                "lily_pad" => "waterlily".into(),
                "knowledge_book" => "book".into(),
                "magma_block" => "magma".into(),
                "map" => "empty_map".into(),
                "melon" => "melon_block".into(),
                "music_disc_bounce" => "music_disc_13".into(),
                "nether_brick" => "netherbrick".into(),
                "nether_bricks" => "nether_brick".into(),
                "nether_quartz_ore" => "quartz_ore".into(),
                "note_block" => "noteblock".into(),
                "oak_button" => "wooden_button".into(),
                "oak_door" => "wooden_door".into(),
                "oak_fence_gate" => "fence_gate".into(),
                "oak_pressure_plate" => "wooden_pressure_plate".into(),
                "oak_trapdoor" => "trapdoor".into(),
                "powered_rail" => "golden_rail".into(),
                "prismarine_brick_stairs" => "prismarine_bricks_stairs".into(),
                "red_nether_bricks" => "red_nether_brick".into(),
                "rooted_dirt" => "dirt_with_roots".into(),
                "scute" => "turtle_scute".into(),
                "shulker_box" => "undyed_shulker_box".into(),
                "slime_block" => "slime".into(),
                "small_dripleaf" => "small_dripleaf_block".into(),
                "snow" => "snow_layer".into(),
                "snow_block" => "snow".into(),
                "spawner" => "mob_spawner".into(),
                "stonecutter" => "stonecutter_block".into(),
                "stone_stairs" => "normal_stone_stairs".into(),
                "terracotta" => "hardened_clay".into(),
                "test_block" => "unknown".into(),
                "test_instance_block" => "unknown".into(),
                "tipped_arrow" | "spectral_arrow" => "arrow".into(),
                "waxed_copper_block" => "waxed_copper".into(),
                "zombified_piglin_spawn_egg" => "zombie_pigman_spawn_egg".into(),
                n if n.contains("sulfur") => "unknown".into(),
                n => {
                    if n.ends_with("banner") {
                        "banner".into()
                    } else if n.ends_with("bed") {
                        "bed".into()
                    } else if n.starts_with("stone_slab") {
                        n.replace("stone_slab", "stone_block_slab")
                    } else if n.starts_with("double_stone_slab") {
                        n.replace("double_stone_slab", "stone_block_slab")
                    } else if n.ends_with("_map") {
                        let candidate = format!("minecraft:{n}");
                        if be_valid_item_identifiers.contains(&candidate) {
                            n.into()
                        } else {
                            "filled_map".into()
                        }
                    } else {
                        n.into()
                    }
                }
            };

        // THIS COERCES CODEGEN FOR JAVA 26.3
        // This is for coercing the codegen into working for java 26.3 temporarily.
        /// Remove the if statement and replace with the assert statement below.
        /// replace with:
        /// assert!(
        ///     be_valid_item_identifiers.contains(&be_identifier),
        ///     "Invalid Bedrock identifier `{be_identifier}`. From Java name `{name}`"
        /// );
        if !be_valid_item_identifiers.contains(&be_identifier) {
            be_identifier = "minecraft:unknown".into();
        }

        let block = item_to_block.get(&item.id);

        let type_suffix = name
            .split(':')
            .next_back()
            .unwrap()
            .split('_')
            .next_back()
            .unwrap();

        be_item_remaps.insert(
            name,
            BedrockRemapEntry {
                identifier: be_identifier,
                data: *be_item_data_overrides.get(name).unwrap_or(&0),
                block_runtime_id_range: block.and_then(|b| {
                    let min = b.states.first();
                    let max = b.states.last();

                    if let (Some(min), Some(max)) = (min, max) {
                        Some((min.id.0 as i32, max.id.0 as i32))
                    } else {
                        None
                    }
                }),
                tool_type: match type_suffix {
                    "sword" => Some(BedrockToolType::Sword),
                    "shovel" => Some(BedrockToolType::Shovel),
                    "pickaxe" => Some(BedrockToolType::Pickaxe),
                    "axe" => Some(BedrockToolType::Axe),
                    "shears" => Some(BedrockToolType::Shears),
                    "hoe" => Some(BedrockToolType::Hoe),
                    _ => None,
                },
                armor_type: match type_suffix {
                    "helmet" => Some(BedrockArmorType::Helmet),
                    "chestplate" => Some(BedrockArmorType::Chestplate),
                    "leggings" => Some(BedrockArmorType::Leggings),
                    "boots" => Some(BedrockArmorType::Boots),
                    _ => None,
                },
                protection_value: if let (Some(_), Some(_)) = (
                    &item.components.equippable,
                    &item.components.attribute_modifiers,
                ) {
                    Some(1) // TODO: calculate correct value
                } else {
                    None
                },
                is_entity_placer: item.components.entity_data.is_some()
                    || matches!(
                        &**name,
                        "firework_rocket"
                            | "minecart"
                            | "tnt_minecart"
                            | "chest_minecart"
                            | "hopper_minecart"
                            | "furnace_minecart"
                            | "command_block_minecart"
                            | "oak_boat"
                            | "oak_chest_boat"
                            | "spruce_boat"
                            | "spruce_chest_boat"
                            | "birch_boat"
                            | "birch_chest_boat"
                            | "jungle_boat"
                            | "jungle_chest_boat"
                            | "acacia_boat"
                            | "acacia_chest_boat"
                            | "dark_oak_boat"
                            | "dark_oak_chest_boat"
                            | "mangrove_boat"
                            | "mangrove_chest_boat"
                            | "cherry_boat"
                            | "cherry_chest_boat"
                            | "pale_oak_boat"
                            | "pale_oak_chest_boat"
                            | "bamboo_raft"
                            | "bamboo_chest_raft"
                    ),
                is_edible: item.components.food.is_some(),
            },
        );
    }

    let mut bedrock_item_definitions = Vec::new();
    for item in be_runtime_item_states {
        let components = be_item_components.get(&item.name).cloned().flatten();

        bedrock_item_definitions.push(BedrockItem {
            id: item.id,
            registry_key: item.name.clone(),
            version: item.version,
            component_based: item.component_based,
            components: if let Some(c) = components {
                Nbt::new("".to_string(), c.clone())
            } else {
                if item.component_based {
                    eprintln!(
                        "Couldn't find the components for a component based item: {}.",
                        item.name
                    );
                }
                Nbt::new("".to_string(), NbtCompound::new())
            },
        });
    }

    let mut type_from_raw_id_arms = TokenStream::new();
    let mut type_from_name = TokenStream::new();

    let mut constants = TokenStream::new();
    let mut bedrock_constants = TokenStream::new();
    let mut mapping_constants = TokenStream::new();

    let mut all_bedrock_items = Vec::new();

    let mut all_java_items = Vec::new();
    let mut all_java_item_ids = Vec::new();

    for (name, item) in &items {
        let const_ident = format_ident!("{}", name.to_shouty_snake_case());

        let components = &item.components;
        let components_tokens = components.to_token_stream();

        let crafting_remainder = item
            .crafting_remainder
            .as_ref()
            .map(|remainder| {
                let name = remainder
                    .id
                    .strip_prefix("minecraft:")
                    .unwrap_or(&remainder.id);
                let ident = format_ident!("{}", name.to_shouty_snake_case());
                quote! { Some(&Self::#ident) }
            })
            .unwrap_or_else(|| quote! { None });

        let id_lit = LitInt::new(&item.id.to_string(), Span::call_site());

        constants.extend(quote! {
            pub const #const_ident: Self = Self {
                id: #id_lit,
                registry_key: #name,
                crafting_remainder: #crafting_remainder,
                components: &[#components_tokens],
            };
        });

        type_from_raw_id_arms.extend(quote! {
            #id_lit => Some(&Self::#const_ident),
        });

        type_from_name.extend(quote! {
            #name => Some(&Self::#const_ident),
        });

        all_java_items.push(const_ident);
        all_java_item_ids.push(id_lit);
    }

    let bedrock_id_by_key: std::collections::HashMap<String, i16> = bedrock_item_definitions
        .iter()
        .map(|item| (item.registry_key.clone(), item.id))
        .collect();

    for item in bedrock_item_definitions {
        let const_ident = format_ident!(
            "{}",
            item.registry_key
                .split(':')
                .next_back()
                .unwrap()
                .to_shouty_snake_case()
        );

        let id = LitInt::new(&item.id.to_string(), Span::call_site());
        let registry_key = item.registry_key;
        let version = format_ident!(
            "{}",
            match item.version {
                BedrockItemVersion::Legacy => "Legacy",
                BedrockItemVersion::DataDriven => "DataDriven",
                BedrockItemVersion::None => "None",
            }
        );
        let component_based = item.component_based;

        let components_bytes_lit =
            LitByteStr::new(&item.components.write_bedrock(), Span::call_site());

        bedrock_constants.extend(quote! {
            pub const #const_ident: Self = Self {
                id: #id,
                registry_key: #registry_key,
                version: BedrockItemVersion::#version,
                component_based: #component_based,
                definition_components: #components_bytes_lit
            };
        });

        all_bedrock_items.push(const_ident);
    }
    let all_bedrock_items_count_lit =
        LitInt::new(&all_bedrock_items.len().to_string(), Span::call_site());

    for (java_name, mapping) in &be_item_remaps {
        let java_const_ident = format_ident!("{}", java_name.to_shouty_snake_case());
        let bedrock_const_ident = format_ident!(
            "{}",
            mapping
                .identifier
                .split(':')
                .next_back()
                .unwrap()
                .to_shouty_snake_case()
        );

        let data_lit = LitInt::new(&mapping.data.to_string(), Span::call_site());

        mapping_constants.extend(quote! {
            pub const #java_const_ident: Self = Self {
                java_item: &Item::#java_const_ident,

                bedrock_item: &BedrockItem::#bedrock_const_ident,
                bedrock_data: #data_lit,
                bedrock_block_state: 0
            };
        });
    }

    let mut bedrock_to_java_arms = TokenStream::new();
    let mut seen_bedrock_pairs = std::collections::HashSet::new();

    let mut bedrock_id_fallback_arms = TokenStream::new();
    let mut seen_bedrock_ids = std::collections::HashSet::new();

    for (java_name, mapping) in &be_item_remaps {
        let java_const_ident = format_ident!("{}", java_name.to_shouty_snake_case());
        let bedrock_id = *bedrock_id_by_key
            .get(&mapping.identifier)
            .unwrap_or_else(|| {
                panic!(
                    "Missing bedrock item for identifier: {}",
                    mapping.identifier
                )
            });
        let bedrock_data = mapping.data;

        if seen_bedrock_pairs.insert((bedrock_id, bedrock_data)) {
            let id_lit = syn::LitInt::new(&bedrock_id.to_string(), proc_macro2::Span::call_site());
            let data_lit =
                syn::LitInt::new(&bedrock_data.to_string(), proc_macro2::Span::call_site());
            bedrock_to_java_arms.extend(quote! {
                (#id_lit, #data_lit) => Some(&Self::#java_const_ident),
            });
        }

        if seen_bedrock_ids.insert(bedrock_id) {
            let id_lit = syn::LitInt::new(&bedrock_id.to_string(), proc_macro2::Span::call_site());
            bedrock_id_fallback_arms.extend(quote! {
                #id_lit => Some(&Self::#java_const_ident),
            });
        }
    }

    quote! {
        #[allow(clippy::wildcard_imports, clippy::enum_glob_use, clippy::too_many_lines)]
        use crate::data_component::DataComponent::*;
        use crate::data_component_impl::*;
        use crate::tag::{RegistryKey, Taggable};
        use pumpkin_util::text::TextComponent;
        use std::borrow::Cow;
        use std::hash::{Hash, Hasher};
        use crate::{tag, AttributeModifierSlot};
        use crate::attributes::Attributes;
        use crate::data_component_impl::IDSet::{IDs, Tag};
        use crate::data_component::DataComponent;
        use crate::effect::StatusEffect;
        use crate::Block;
        use crate::sound::Sound;

        use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};

        #[derive(Clone)]
        pub struct Item {
            pub id: u16,
            pub registry_key: &'static str,
            pub crafting_remainder: Option<&'static Item>,
            pub components: &'static [(DataComponent, &'static dyn DataComponentImpl)],
        }

        impl Eq for Item {}

        impl Hash for Item {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.id.hash(state);
            }
        }

        impl PartialEq for Item {
            fn eq(&self, other: &Self) -> bool {
                self.id == other.id
            }
        }

        impl std::fmt::Debug for Item {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct("Item")
                    .field("id", &self.id)
                    .field("registry_key", &self.registry_key)
                    .finish()
            }
        }

        #[derive(Clone)]
        pub enum BedrockItemVersion {
            Legacy,
            DataDriven,
            None
        }

        #[derive(Clone)]
        pub struct BedrockItem {
            pub id: i16,
            pub registry_key: &'static str,
            pub version: BedrockItemVersion,
            pub component_based: bool,
            pub definition_components: &'static [u8],
        }

        impl Eq for BedrockItem {}

        impl Hash for BedrockItem {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.id.hash(state);
            }
        }

        impl PartialEq for BedrockItem {
            fn eq(&self, other: &Self) -> bool {
                self.id == other.id
            }
        }

        #[derive(Clone)]
        pub struct JavaToBedrockItemMapping {
            pub java_item: &'static Item,

            pub bedrock_item: &'static BedrockItem,
            pub bedrock_data: u32,
            pub bedrock_block_state: u16
        }

        impl Eq for JavaToBedrockItemMapping {}

        impl Hash for JavaToBedrockItemMapping {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.java_item.id.hash(state);
            }
        }

        impl PartialEq for JavaToBedrockItemMapping {
            fn eq(&self, other: &Self) -> bool {
                self.java_item.id == other.java_item.id
            }
        }

        impl Item {
            #constants

            #[must_use]
            #[allow(deprecated)]
            pub fn translated_name(&self) -> TextComponent {
                let name = self
                    .components
                    .iter()
                    .find_map(|(id, data)| {
                        if id == &ItemName {
                            data.as_any()
                                .downcast_ref::<ItemNameImpl>()
                                .map(|name| name.name.as_ref())
                        } else {
                            None
                        }
                    })
                    .unwrap_or(self.registry_key);
                TextComponent::translate(name, &[])
            }

            #[doc = "Try to parse an item from a resource location string."]
            #[must_use]
            pub fn from_registry_key(name: &str) -> Option<&'static Self> {
                let name = name.strip_prefix("minecraft:").unwrap_or(name);
                match name {
                    #type_from_name
                    _ => None
                }
            }

            #[doc = "Try to parse an item from a raw id."]
            #[must_use]
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

        impl BedrockItem {
            #bedrock_constants

            pub const ALL_BEDROCK_ITEMS: [&'static Self; #all_bedrock_items_count_lit] = [
                #( &Self::#all_bedrock_items, )*
            ];
        }

        impl JavaToBedrockItemMapping {
            #mapping_constants

            #[must_use]
            pub const fn from_java_item_id(item_id: u16) -> Option<&'static Self> {
                match item_id {
                    #( #all_java_item_ids => Some(&Self::#all_java_items), )*
                    _ => None,
                }
            }

            #[must_use]
            pub fn from_bedrock(bedrock_id: i16, bedrock_data: u32) -> Option<&'static Self> {
                match (bedrock_id, bedrock_data) {
                    #bedrock_to_java_arms
                    _ => match bedrock_id {
                        #bedrock_id_fallback_arms
                        _ => None,
                    }
                }
            }
        }
    }
}
