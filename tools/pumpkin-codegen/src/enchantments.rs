use std::{collections::BTreeMap, fs};

use heck::{ToPascalCase, ToShoutySnakeCase};
use proc_macro2::TokenStream;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::TextContent::Translate;
use quote::{format_ident, quote};
use serde::Deserialize;

/// Raw deserialization shape for a single enchantment entry from `enchantment/*.json`.
#[derive(Deserialize)]
pub struct Enchantment {
    /// Numeric registry ID for this enchantment.
    #[serde(default)]
    pub id: u8,
    /// Anvil repair cost multiplier added when applying this enchantment.
    pub anvil_cost: u32,
    /// Tag path (prefixed with `#`) of items that support this enchantment.
    pub supported_items: String,
    /// Display name component for this enchantment (typically a translation key).
    pub description: TextComponent,
    /// Optional exclusive-set tag; enchantments in the same set are mutually incompatible.
    pub exclusive_set: Option<String>,
    /// Maximum level this enchantment can reach.
    pub max_level: i32,
    /// Equipment slots this enchantment's attribute modifiers apply to.
    pub slots: Vec<AttributeModifierSlot>,
    /// The weight of this enchantment (used for random selection).
    pub weight: i32,
    /// The minimum cost to get this enchantment.
    pub min_cost: Cost,
    /// The maximum cost to get this enchantment.
    pub max_cost: Cost,
    /// Data-driven effects map.
    #[serde(default)]
    pub effects: serde_json::Map<String, serde_json::Value>,
}

#[derive(Deserialize, Clone, Copy)]
pub struct Cost {
    pub base: i32,
    pub per_level_above_first: i32,
}

/// Equipment slot category that an enchantment's attribute modifier applies to.
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
    /// Converts this slot variant into a `TokenStream` for use in generated code.
    pub fn to_tokens(&self) -> TokenStream {
        match self {
            Self::Any => quote! { AttributeModifierSlot::Any },
            Self::MainHand => quote! { AttributeModifierSlot::MainHand },
            Self::OffHand => quote! { AttributeModifierSlot::OffHand },
            Self::Hand => quote! { AttributeModifierSlot::Hand },
            Self::Feet => quote! { AttributeModifierSlot::Feet },
            Self::Legs => quote! { AttributeModifierSlot::Legs },
            Self::Chest => quote! { AttributeModifierSlot::Chest },
            Self::Head => quote! { AttributeModifierSlot::Head },
            Self::Armor => quote! { AttributeModifierSlot::Armor },
            Self::Body => quote! { AttributeModifierSlot::Body },
            Self::Saddle => quote! { AttributeModifierSlot::Saddle },
        }
    }
}

fn parse_level_based_value(val: &serde_json::Value) -> TokenStream {
    if let Some(num) = val.as_f64() {
        let num_f32 = num as f32;
        return quote! { LevelBasedValue::Constant(#num_f32) };
    }

    if let Some(obj) = val.as_object() {
        let val_type = obj.get("type").and_then(|t| t.as_str()).unwrap_or("");
        match val_type {
            "minecraft:linear" => {
                let base = obj.get("base").and_then(|b| b.as_f64()).unwrap_or(0.0) as f32;
                let per_level = obj
                    .get("per_level_above_first")
                    .and_then(|p| p.as_f64())
                    .unwrap_or(0.0) as f32;
                quote! {
                    LevelBasedValue::Linear {
                        base: #base,
                        per_level_above_first: #per_level,
                    }
                }
            }
            "minecraft:clamped" => {
                let inner =
                    parse_level_based_value(obj.get("value").unwrap_or(&serde_json::Value::Null));
                let min = obj.get("min").and_then(|m| m.as_f64()).unwrap_or(0.0) as f32;
                let max = obj.get("max").and_then(|m| m.as_f64()).unwrap_or(0.0) as f32;
                quote! {
                    LevelBasedValue::Clamped {
                        value: &#inner,
                        min: #min,
                        max: #max,
                    }
                }
            }
            "minecraft:fraction" => {
                let num = parse_level_based_value(
                    obj.get("numerator").unwrap_or(&serde_json::Value::Null),
                );
                let denom = parse_level_based_value(
                    obj.get("denominator").unwrap_or(&serde_json::Value::Null),
                );
                quote! {
                    LevelBasedValue::Fraction {
                        numerator: &#num,
                        denominator: &#denom,
                    }
                }
            }
            "minecraft:levels_squared" => {
                let added = obj.get("added").and_then(|a| a.as_f64()).unwrap_or(0.0) as f32;
                quote! {
                    LevelBasedValue::LevelsSquared { added: #added }
                }
            }
            "minecraft:lookup" => {
                let values: Vec<f32> = obj
                    .get("values")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|x| x.as_f64().map(|f| f as f32))
                            .collect()
                    })
                    .unwrap_or_default();
                let fallback = parse_level_based_value(
                    obj.get("fallback").unwrap_or(&serde_json::Value::Null),
                );
                quote! {
                    LevelBasedValue::Lookup {
                        values: &[#(#values),*],
                        fallback: &#fallback,
                    }
                }
            }
            _ => quote! { LevelBasedValue::Constant(0.0) },
        }
    } else {
        quote! { LevelBasedValue::Constant(0.0) }
    }
}

fn parse_particle(v: &serde_json::Value) -> TokenStream {
    let type_str = if let Some(s) = v.as_str() {
        s
    } else {
        v.get("type").and_then(|s| s.as_str()).unwrap_or("")
    };
    let raw = type_str.strip_prefix("minecraft:").unwrap_or(type_str);
    let ident = format_ident!("{}", raw.to_pascal_case());
    quote! { crate::particle::Particle::#ident }
}

fn parse_position_source(v: Option<&serde_json::Value>) -> (TokenStream, f32, f32) {
    let obj = v.and_then(|val| val.as_object());
    let type_str = obj
        .and_then(|o| o.get("type"))
        .and_then(|s| s.as_str())
        .unwrap_or("entity_position");
    let offset = obj
        .and_then(|o| o.get("offset"))
        .and_then(|n| n.as_f64())
        .unwrap_or(0.0) as f32;
    let scale = obj
        .and_then(|o| o.get("scale"))
        .and_then(|n| n.as_f64())
        .unwrap_or(1.0) as f32;
    let type_token = match type_str {
        "in_bounding_box" => quote! { PositionSourceType::InBoundingBox },
        _ => quote! { PositionSourceType::EntityPosition },
    };
    (type_token, offset, scale)
}

fn parse_float_provider(v: &serde_json::Value) -> TokenStream {
    if let Some(f) = v.as_f64() {
        let f = f as f32;
        quote! { pumpkin_util::math::float_provider::FloatProvider::Constant(#f) }
    } else if let Some(type_str) = v.get("type").and_then(|s| s.as_str()) {
        match type_str {
            "minecraft:constant" => {
                let val = v.get("value").and_then(|n| n.as_f64()).unwrap_or(0.0) as f32;
                quote! { pumpkin_util::math::float_provider::FloatProvider::Constant(#val) }
            }
            "minecraft:uniform" => {
                let min = v
                    .get("min_inclusive")
                    .and_then(|n| n.as_f64())
                    .unwrap_or(0.0) as f32;
                let max = v
                    .get("max_exclusive")
                    .and_then(|n| n.as_f64())
                    .unwrap_or(0.0) as f32;
                quote! {
                    pumpkin_util::math::float_provider::FloatProvider::Object(
                        pumpkin_util::math::float_provider::NormalFloatProvider::Uniform(
                            pumpkin_util::math::float_provider::UniformFloatProvider::new(#min, #max)
                        )
                    )
                }
            }
            "minecraft:trapezoid" => {
                let min = v.get("min").and_then(|n| n.as_f64()).unwrap_or(0.0) as f32;
                let max = v.get("max").and_then(|n| n.as_f64()).unwrap_or(0.0) as f32;
                let plateau = v.get("plateau").and_then(|n| n.as_f64()).unwrap_or(0.0) as f32;
                quote! {
                    pumpkin_util::math::float_provider::FloatProvider::Object(
                        pumpkin_util::math::float_provider::NormalFloatProvider::Trapezoid(
                            pumpkin_util::math::float_provider::TrapezoidFloatProvider::new(#min, #max, #plateau)
                        )
                    )
                }
            }
            _ => quote! { pumpkin_util::math::float_provider::FloatProvider::Constant(0.0) },
        }
    } else {
        quote! { pumpkin_util::math::float_provider::FloatProvider::Constant(0.0) }
    }
}

fn parse_velocity_source(v: Option<&serde_json::Value>) -> (f32, TokenStream) {
    let obj = v.and_then(|val| val.as_object());
    let movement_scale = obj
        .and_then(|o| o.get("movement_scale"))
        .and_then(|n| n.as_f64())
        .unwrap_or(0.0) as f32;
    let base = obj.and_then(|o| o.get("base"));
    let base_token = match base {
        Some(b) => parse_float_provider(b),
        None => quote! { pumpkin_util::math::float_provider::FloatProvider::Constant(0.0) },
    };
    (movement_scale, base_token)
}

fn parse_vec3(v: Option<&serde_json::Value>) -> (f64, f64, f64) {
    let arr = v.and_then(|val| val.as_array());
    if let Some(a) = arr {
        let x = a.first().and_then(|n| n.as_f64()).unwrap_or(0.0);
        let y = a.get(1).and_then(|n| n.as_f64()).unwrap_or(0.0);
        let z = a.get(2).and_then(|n| n.as_f64()).unwrap_or(0.0);
        (x, y, z)
    } else {
        (0.0, 0.0, 0.0)
    }
}

fn parse_status_effect_list(v: Option<&serde_json::Value>) -> TokenStream {
    match v {
        Some(serde_json::Value::String(s)) => {
            let raw = s.strip_prefix("minecraft:").unwrap_or(s);
            let ident = format_ident!("{}", raw.to_shouty_snake_case());
            quote! { &[&crate::effect::StatusEffect::#ident] }
        }
        Some(serde_json::Value::Array(arr)) => {
            let idents: Vec<_> = arr
                .iter()
                .filter_map(|x| x.as_str())
                .map(|s| {
                    let raw = s.strip_prefix("minecraft:").unwrap_or(s);
                    let ident = format_ident!("{}", raw.to_shouty_snake_case());
                    quote! { &crate::effect::StatusEffect::#ident }
                })
                .collect();
            quote! { &[#(#idents),*] }
        }
        _ => quote! { &[] },
    }
}

fn parse_entity_effect(val: &serde_json::Value) -> TokenStream {
    if let Some(obj) = val.as_object() {
        let eff_type = obj.get("type").and_then(|t| t.as_str()).unwrap_or("");
        match eff_type {
            "minecraft:ignite" => {
                let dur = parse_level_based_value(
                    obj.get("duration").unwrap_or(&serde_json::Value::Null),
                );
                quote! { EnchantmentEntityEffect::Ignite { duration: #dur } }
            }
            "minecraft:damage_entity" => {
                let min_damage = parse_level_based_value(
                    obj.get("min_damage")
                        .or_else(|| obj.get("damage"))
                        .unwrap_or(&serde_json::Value::Null),
                );
                let max_damage = parse_level_based_value(
                    obj.get("max_damage")
                        .or_else(|| obj.get("damage"))
                        .unwrap_or(&serde_json::Value::Null),
                );
                let damage_type = obj.get("damage_type").and_then(|s| s.as_str());
                let damage_type_token = match damage_type {
                    Some(dt) => {
                        let dt_raw = dt.strip_prefix("minecraft:").unwrap_or(dt);
                        let dt_ident = format_ident!("{}", dt_raw.to_shouty_snake_case());
                        quote! { Some(&crate::damage::DamageType::#dt_ident) }
                    }
                    None => quote! { None },
                };
                quote! {
                    EnchantmentEntityEffect::DamageEntity {
                        min_damage: #min_damage,
                        max_damage: #max_damage,
                        damage_type: #damage_type_token,
                    }
                }
            }
            "minecraft:change_item_damage" => {
                let amount =
                    parse_level_based_value(obj.get("amount").unwrap_or(&serde_json::Value::Null));
                quote! {
                    EnchantmentEntityEffect::ChangeItemDamage {
                        amount: #amount,
                    }
                }
            }
            "minecraft:play_sound" => {
                let sound = obj.get("sound").and_then(|s| s.as_str()).unwrap_or("");
                quote! { EnchantmentEntityEffect::PlaySound { sound: #sound } }
            }
            "minecraft:replace_block" => {
                let offset_x = obj
                    .get("offset")
                    .and_then(|v| v.get(0))
                    .and_then(|n| n.as_i64())
                    .unwrap_or(0) as i32;
                let offset_y = obj
                    .get("offset")
                    .and_then(|v| v.get(1))
                    .and_then(|n| n.as_i64())
                    .unwrap_or(0) as i32;
                let offset_z = obj
                    .get("offset")
                    .and_then(|v| v.get(2))
                    .and_then(|n| n.as_i64())
                    .unwrap_or(0) as i32;
                let event = obj.get("trigger_game_event").and_then(|s| s.as_str());
                let event_token = match event {
                    Some(ev) => {
                        let ev_raw = ev.strip_prefix("minecraft:").unwrap_or(ev);
                        let ev_ident = format_ident!("{}", ev_raw.to_pascal_case());
                        quote! { Some(crate::game_event::GameEvent::#ev_ident) }
                    }
                    None => quote! { None },
                };
                quote! {
                    EnchantmentEntityEffect::ReplaceBlock {
                        offset_x: #offset_x,
                        offset_y: #offset_y,
                        offset_z: #offset_z,
                        trigger_game_event: #event_token,
                    }
                }
            }
            "minecraft:set_block_properties" => {
                let offset_x = obj
                    .get("offset")
                    .and_then(|v| v.get(0))
                    .and_then(|n| n.as_i64())
                    .unwrap_or(0) as i32;
                let offset_y = obj
                    .get("offset")
                    .and_then(|v| v.get(1))
                    .and_then(|n| n.as_i64())
                    .unwrap_or(0) as i32;
                let offset_z = obj
                    .get("offset")
                    .and_then(|v| v.get(2))
                    .and_then(|n| n.as_i64())
                    .unwrap_or(0) as i32;
                let event = obj.get("trigger_game_event").and_then(|s| s.as_str());
                let event_token = match event {
                    Some(ev) => {
                        let ev_raw = ev.strip_prefix("minecraft:").unwrap_or(ev);
                        let ev_ident = format_ident!("{}", ev_raw.to_pascal_case());
                        quote! { Some(crate::game_event::GameEvent::#ev_ident) }
                    }
                    None => quote! { None },
                };
                let properties_obj = obj.get("properties").and_then(|p| p.as_object());
                let properties_tokens: Vec<_> = properties_obj
                    .map(|o| {
                        o.iter()
                            .map(|(k, v)| {
                                let val_str = v.as_str().unwrap_or("");
                                quote! { (#k, #val_str) }
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                quote! {
                    EnchantmentEntityEffect::SetBlockProperties {
                        properties: &[#(#properties_tokens),*],
                        offset_x: #offset_x,
                        offset_y: #offset_y,
                        offset_z: #offset_z,
                        trigger_game_event: #event_token,
                    }
                }
            }
            "minecraft:replace_disk" => {
                let radius =
                    parse_level_based_value(obj.get("radius").unwrap_or(&serde_json::Value::Null));
                let height =
                    parse_level_based_value(obj.get("height").unwrap_or(&serde_json::Value::Null));
                let offset_x = obj
                    .get("offset")
                    .and_then(|v| v.get(0))
                    .and_then(|n| n.as_i64())
                    .unwrap_or(0) as i32;
                let offset_y = obj
                    .get("offset")
                    .and_then(|v| v.get(1))
                    .and_then(|n| n.as_i64())
                    .unwrap_or(0) as i32;
                let offset_z = obj
                    .get("offset")
                    .and_then(|v| v.get(2))
                    .and_then(|n| n.as_i64())
                    .unwrap_or(0) as i32;
                let event = obj.get("trigger_game_event").and_then(|s| s.as_str());
                let event_token = match event {
                    Some(ev) => {
                        let ev_raw = ev.strip_prefix("minecraft:").unwrap_or(ev);
                        let ev_ident = format_ident!("{}", ev_raw.to_pascal_case());
                        quote! { Some(crate::game_event::GameEvent::#ev_ident) }
                    }
                    None => quote! { None },
                };
                let block_state_token = obj.get("block_state").map_or_else(
                    || quote! { crate::block::Block::FROSTED_ICE.default_state },
                    parse_block_state,
                );
                let predicate_token = obj
                    .get("predicate")
                    .map_or_else(|| quote! { None }, parse_replace_disk_predicate);
                quote! {
                    EnchantmentEntityEffect::ReplaceDisk {
                        radius: #radius,
                        height: #height,
                        offset_x: #offset_x,
                        offset_y: #offset_y,
                        offset_z: #offset_z,
                        predicate: #predicate_token,
                        block_state: #block_state_token,
                        trigger_game_event: #event_token,
                    }
                }
            }
            "minecraft:summon_entity" => {
                let entity_val = obj.get("entity");
                let entity_token = match entity_val {
                    Some(serde_json::Value::String(s)) => {
                        let raw = s.strip_prefix("minecraft:").unwrap_or(s);
                        let ident = format_ident!("{}", raw.to_shouty_snake_case());
                        quote! { &[&crate::entity::EntityType::#ident] }
                    }
                    Some(serde_json::Value::Array(arr)) => {
                        let idents: Vec<TokenStream> = arr
                            .iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| {
                                let raw = s.strip_prefix("minecraft:").unwrap_or(s);
                                let ident = format_ident!("{}", raw.to_shouty_snake_case());
                                quote! { &crate::entity::EntityType::#ident }
                            })
                            .collect();
                        quote! { &[#(#idents),*] }
                    }
                    _ => quote! { &[] },
                };
                let join_team = obj
                    .get("join_team")
                    .and_then(|b| b.as_bool())
                    .unwrap_or(false);
                quote! {
                    EnchantmentEntityEffect::SummonEntity {
                        entity_types: #entity_token,
                        join_team: #join_team,
                    }
                }
            }
            "minecraft:spawn_particles" => {
                let particle_val = obj.get("particle").unwrap_or(&serde_json::Value::Null);
                let particle_token = parse_particle(particle_val);
                let (h_pos_type, h_pos_off, h_pos_scale) =
                    parse_position_source(obj.get("horizontal_position"));
                let (v_pos_type, v_pos_off, v_pos_scale) =
                    parse_position_source(obj.get("vertical_position"));
                let (h_vel_scale, h_vel_base) =
                    parse_velocity_source(obj.get("horizontal_velocity"));
                let (v_vel_scale, v_vel_base) = parse_velocity_source(obj.get("vertical_velocity"));
                let speed_token =
                    parse_float_provider(obj.get("speed").unwrap_or(&serde_json::Value::Null));

                quote! {
                    EnchantmentEntityEffect::SpawnParticles {
                        particle: #particle_token,
                        horizontal_position: PositionSource {
                            source_type: #h_pos_type,
                            offset: #h_pos_off,
                            scale: #h_pos_scale,
                        },
                        vertical_position: PositionSource {
                            source_type: #v_pos_type,
                            offset: #v_pos_off,
                            scale: #v_pos_scale,
                        },
                        horizontal_velocity: VelocitySource {
                            movement_scale: #h_vel_scale,
                            base: #h_vel_base,
                        },
                        vertical_velocity: VelocitySource {
                            movement_scale: #v_vel_scale,
                            base: #v_vel_base,
                        },
                        speed: #speed_token,
                    }
                }
            }
            "minecraft:run_function" => {
                let function = obj.get("function").and_then(|s| s.as_str()).unwrap_or("");
                quote! {
                    EnchantmentEntityEffect::RunFunction {
                        function: #function,
                    }
                }
            }
            "minecraft:apply_exhaustion" => {
                let amount =
                    parse_level_based_value(obj.get("amount").unwrap_or(&serde_json::Value::Null));
                quote! {
                    EnchantmentEntityEffect::ApplyExhaustion {
                        amount: #amount,
                    }
                }
            }
            "minecraft:apply_impulse" => {
                let (dir_x, dir_y, dir_z) = parse_vec3(obj.get("direction"));
                let (scale_x, scale_y, scale_z) = parse_vec3(obj.get("coordinate_scale"));
                let mag = parse_level_based_value(
                    obj.get("magnitude").unwrap_or(&serde_json::Value::Null),
                );
                quote! {
                    EnchantmentEntityEffect::ApplyImpulse {
                        direction: pumpkin_util::math::vector3::Vector3::new(#dir_x, #dir_y, #dir_z),
                        coordinate_scale: pumpkin_util::math::vector3::Vector3::new(#scale_x, #scale_y, #scale_z),
                        magnitude: #mag,
                    }
                }
            }
            "minecraft:apply_mob_effect" => {
                let to_apply = parse_status_effect_list(obj.get("to_apply"));
                let min_duration = parse_level_based_value(
                    obj.get("min_duration").unwrap_or(&serde_json::Value::Null),
                );
                let max_duration = parse_level_based_value(
                    obj.get("max_duration").unwrap_or(&serde_json::Value::Null),
                );
                let min_amplifier = parse_level_based_value(
                    obj.get("min_amplifier").unwrap_or(&serde_json::Value::Null),
                );
                let max_amplifier = parse_level_based_value(
                    obj.get("max_amplifier").unwrap_or(&serde_json::Value::Null),
                );
                quote! {
                    EnchantmentEntityEffect::ApplyMobEffect {
                        to_apply: #to_apply,
                        min_duration: #min_duration,
                        max_duration: #max_duration,
                        min_amplifier: #min_amplifier,
                        max_amplifier: #max_amplifier,
                    }
                }
            }
            "minecraft:explode" => {
                let attribute_to_user = obj
                    .get("attribute_to_user")
                    .and_then(|b| b.as_bool())
                    .unwrap_or(false);
                let damage_type = obj.get("damage_type").and_then(|s| s.as_str());
                let damage_type_token = match damage_type {
                    Some(dt) => {
                        let dt_raw = dt.strip_prefix("minecraft:").unwrap_or(dt);
                        let dt_ident = format_ident!("{}", dt_raw.to_shouty_snake_case());
                        quote! { Some(&crate::damage::DamageType::#dt_ident) }
                    }
                    None => quote! { None },
                };
                let knockback_multiplier =
                    obj.get("knockback_multiplier").map(parse_level_based_value);
                let knockback_token = match knockback_multiplier {
                    Some(kb) => quote! { Some(#kb) },
                    None => quote! { None },
                };
                let immune_blocks = obj.get("immune_blocks").and_then(|s| s.as_str());
                let immune_blocks_token = match immune_blocks {
                    Some(ib) => quote! { Some(#ib) },
                    None => quote! { None },
                };
                let offset_x = obj
                    .get("offset")
                    .and_then(|v| v.get(0))
                    .and_then(|n| n.as_f64())
                    .unwrap_or(0.0);
                let offset_y = obj
                    .get("offset")
                    .and_then(|v| v.get(1))
                    .and_then(|n| n.as_f64())
                    .unwrap_or(0.0);
                let offset_z = obj
                    .get("offset")
                    .and_then(|v| v.get(2))
                    .and_then(|n| n.as_f64())
                    .unwrap_or(0.0);
                let radius =
                    parse_level_based_value(obj.get("radius").unwrap_or(&serde_json::Value::Null));
                let create_fire = obj
                    .get("create_fire")
                    .and_then(|b| b.as_bool())
                    .unwrap_or(false);
                let block_interaction = obj
                    .get("block_interaction")
                    .and_then(|s| s.as_str())
                    .unwrap_or("none");
                let small_particle = obj.get("small_particle").and_then(|p| {
                    p.get("type")
                        .and_then(|t| t.as_str())
                        .or_else(|| p.as_str())
                });
                let small_particle_token = match small_particle {
                    Some(p) => {
                        let p_raw = p.strip_prefix("minecraft:").unwrap_or(p);
                        let p_ident = format_ident!("{}", p_raw.to_pascal_case());
                        quote! { Some(crate::particle::Particle::#p_ident) }
                    }
                    None => quote! { None },
                };
                let large_particle = obj.get("large_particle").and_then(|p| {
                    p.get("type")
                        .and_then(|t| t.as_str())
                        .or_else(|| p.as_str())
                });
                let large_particle_token = match large_particle {
                    Some(p) => {
                        let p_raw = p.strip_prefix("minecraft:").unwrap_or(p);
                        let p_ident = format_ident!("{}", p_raw.to_pascal_case());
                        quote! { Some(crate::particle::Particle::#p_ident) }
                    }
                    None => quote! { None },
                };
                let sound = obj.get("sound").and_then(|s| s.as_str());
                let sound_token = match sound {
                    Some(s) => {
                        let s_raw = s.strip_prefix("minecraft:").unwrap_or(s);
                        let s_ident = format_ident!("{}", s_raw.to_pascal_case());
                        quote! { Some(crate::sound::Sound::#s_ident) }
                    }
                    None => quote! { None },
                };

                quote! {
                    EnchantmentEntityEffect::Explode {
                        attribute_to_user: #attribute_to_user,
                        damage_type: #damage_type_token,
                        knockback_multiplier: #knockback_token,
                        immune_blocks: #immune_blocks_token,
                        offset_x: #offset_x,
                        offset_y: #offset_y,
                        offset_z: #offset_z,
                        radius: #radius,
                        create_fire: #create_fire,
                        block_interaction: #block_interaction,
                        small_particle: #small_particle_token,
                        large_particle: #large_particle_token,
                        sound: #sound_token,
                    }
                }
            }
            "minecraft:all_of" => {
                let inner: Vec<TokenStream> = obj
                    .get("effects")
                    .and_then(|arr| arr.as_array())
                    .map(|arr| arr.iter().map(parse_entity_effect).collect())
                    .unwrap_or_default();
                quote! { EnchantmentEntityEffect::AllOf(&[#(#inner),*]) }
            }
            _ => quote! { EnchantmentEntityEffect::Other },
        }
    } else {
        quote! { EnchantmentEntityEffect::Other }
    }
}

fn parse_value_effect(val: &serde_json::Value) -> TokenStream {
    if let Some(obj) = val.as_object() {
        let eff_type = obj.get("type").and_then(|t| t.as_str()).unwrap_or("");
        match eff_type {
            "minecraft:add" => {
                let v =
                    parse_level_based_value(obj.get("value").unwrap_or(&serde_json::Value::Null));
                quote! { EnchantmentValueEffect::Add(#v) }
            }
            "minecraft:multiply" => {
                let v = parse_level_based_value(
                    obj.get("factor")
                        .or_else(|| obj.get("value"))
                        .unwrap_or(&serde_json::Value::Null),
                );
                quote! { EnchantmentValueEffect::Multiply(#v) }
            }
            "minecraft:set" => {
                let v =
                    parse_level_based_value(obj.get("value").unwrap_or(&serde_json::Value::Null));
                quote! { EnchantmentValueEffect::Set(#v) }
            }
            "minecraft:remove_binomial" => {
                let v =
                    parse_level_based_value(obj.get("chance").unwrap_or(&serde_json::Value::Null));
                quote! { EnchantmentValueEffect::RemoveBinomial(#v) }
            }
            _ => quote! { EnchantmentValueEffect::Other },
        }
    } else {
        quote! { EnchantmentValueEffect::Other }
    }
}

fn parse_conditional_entity_effects(val: Option<&serde_json::Value>) -> TokenStream {
    let effects: Vec<TokenStream> = val
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|item| {
                    let eff = item.get("effect").unwrap_or(item);
                    let eff_tokens = parse_entity_effect(eff);
                    quote! { ConditionalEffect { effect: #eff_tokens } }
                })
                .collect()
        })
        .unwrap_or_default();
    quote! { &[#(#effects),*] }
}

fn parse_targeted_conditional_entity_effects(val: Option<&serde_json::Value>) -> TokenStream {
    let effects: Vec<TokenStream> = val
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|item| {
                    let eff = item.get("effect").unwrap_or(item);
                    let eff_tokens = parse_entity_effect(eff);
                    let enchanted = match item.get("enchanted").and_then(|e| e.as_str()) {
                        Some("attacker") => quote! { Some(EnchantmentTarget::Attacker) },
                        Some("damaging_entity") => {
                            quote! { Some(EnchantmentTarget::DamagingEntity) }
                        }
                        Some("victim") => quote! { Some(EnchantmentTarget::Victim) },
                        _ => quote! { None },
                    };
                    let affected = match item.get("affected").and_then(|e| e.as_str()) {
                        Some("attacker") => quote! { Some(EnchantmentTarget::Attacker) },
                        Some("damaging_entity") => {
                            quote! { Some(EnchantmentTarget::DamagingEntity) }
                        }
                        Some("victim") => quote! { Some(EnchantmentTarget::Victim) },
                        _ => quote! { None },
                    };
                    quote! {
                        TargetedConditionalEffect {
                            enchanted: #enchanted,
                            affected: #affected,
                            effect: #eff_tokens,
                        }
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    quote! { &[#(#effects),*] }
}

fn parse_conditional_value_effects(val: Option<&serde_json::Value>) -> TokenStream {
    let effects: Vec<TokenStream> = val
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|item| {
                    let eff = item.get("effect").unwrap_or(item);
                    let eff_tokens = parse_value_effect(eff);
                    quote! { ConditionalEffect { effect: #eff_tokens } }
                })
                .collect()
        })
        .unwrap_or_default();
    quote! { &[#(#effects),*] }
}

fn parse_targeted_conditional_value_effects(val: Option<&serde_json::Value>) -> TokenStream {
    let effects: Vec<TokenStream> = val
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|item| {
                    let eff = item.get("effect").unwrap_or(item);
                    let eff_tokens = parse_value_effect(eff);
                    let enchanted = match item.get("enchanted").and_then(|e| e.as_str()) {
                        Some("attacker") => quote! { Some(EnchantmentTarget::Attacker) },
                        Some("damaging_entity") => {
                            quote! { Some(EnchantmentTarget::DamagingEntity) }
                        }
                        Some("victim") => quote! { Some(EnchantmentTarget::Victim) },
                        _ => quote! { None },
                    };
                    let affected = match item.get("affected").and_then(|e| e.as_str()) {
                        Some("attacker") => quote! { Some(EnchantmentTarget::Attacker) },
                        Some("damaging_entity") => {
                            quote! { Some(EnchantmentTarget::DamagingEntity) }
                        }
                        Some("victim") => quote! { Some(EnchantmentTarget::Victim) },
                        _ => quote! { None },
                    };
                    quote! {
                        TargetedConditionalEffect {
                            enchanted: #enchanted,
                            affected: #affected,
                            effect: #eff_tokens,
                        }
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    quote! { &[#(#effects),*] }
}

fn parse_block_state(v: &serde_json::Value) -> TokenStream {
    let state_val = if let Some(state) = v.get("state") {
        state
    } else {
        v
    };
    let name = state_val
        .get("Name")
        .and_then(|n| n.as_str())
        .unwrap_or("minecraft:air");
    let name_stripped = name.strip_prefix("minecraft:").unwrap_or(name);
    let block_ident = format_ident!("{}", name_stripped.to_shouty_snake_case());
    quote! {
        crate::Block::#block_ident.default_state
    }
}

fn parse_replace_disk_predicate(v: &serde_json::Value) -> TokenStream {
    if v.is_null() {
        return quote! { None };
    }
    let pred_token = parse_single_replace_disk_predicate(v);
    quote! { Some(#pred_token) }
}

fn parse_single_replace_disk_predicate(v: &serde_json::Value) -> TokenStream {
    let type_str = v.get("type").and_then(|s| s.as_str()).unwrap_or("");
    match type_str {
        "minecraft:matching_block_tag" => {
            let offset_x = v
                .get("offset")
                .and_then(|a| a.get(0))
                .and_then(|n| n.as_i64())
                .unwrap_or(0) as i32;
            let offset_y = v
                .get("offset")
                .and_then(|a| a.get(1))
                .and_then(|n| n.as_i64())
                .unwrap_or(0) as i32;
            let offset_z = v
                .get("offset")
                .and_then(|a| a.get(2))
                .and_then(|n| n.as_i64())
                .unwrap_or(0) as i32;
            let tag = v.get("tag").and_then(|s| s.as_str()).unwrap_or("");
            let tag_ident = format_ident!("{}", tag.to_uppercase().replace([':', '-'], "_"));
            quote! {
                ReplaceDiskPredicate::MatchingBlockTag {
                    offset: pumpkin_util::math::vector3::Vector3::new(#offset_x, #offset_y, #offset_z),
                    tag: &crate::tag::Block::#tag_ident,
                }
            }
        }
        "minecraft:matching_blocks" => {
            let offset_x = v
                .get("offset")
                .and_then(|a| a.get(0))
                .and_then(|n| n.as_i64())
                .unwrap_or(0) as i32;
            let offset_y = v
                .get("offset")
                .and_then(|a| a.get(1))
                .and_then(|n| n.as_i64())
                .unwrap_or(0) as i32;
            let offset_z = v
                .get("offset")
                .and_then(|a| a.get(2))
                .and_then(|n| n.as_i64())
                .unwrap_or(0) as i32;
            let blocks_token = match v.get("blocks") {
                Some(serde_json::Value::String(s)) => quote! { &[#s] },
                Some(serde_json::Value::Array(arr)) => {
                    let items: Vec<&str> = arr.iter().filter_map(|x| x.as_str()).collect();
                    quote! { &[#(#items),*] }
                }
                _ => quote! { &[] },
            };
            quote! {
                ReplaceDiskPredicate::MatchingBlocks {
                    offset: pumpkin_util::math::vector3::Vector3::new(#offset_x, #offset_y, #offset_z),
                    blocks: #blocks_token,
                }
            }
        }
        "minecraft:matching_fluids" => {
            let offset_x = v
                .get("offset")
                .and_then(|a| a.get(0))
                .and_then(|n| n.as_i64())
                .unwrap_or(0) as i32;
            let offset_y = v
                .get("offset")
                .and_then(|a| a.get(1))
                .and_then(|n| n.as_i64())
                .unwrap_or(0) as i32;
            let offset_z = v
                .get("offset")
                .and_then(|a| a.get(2))
                .and_then(|n| n.as_i64())
                .unwrap_or(0) as i32;
            let fluids_token = match v.get("fluids") {
                Some(serde_json::Value::String(s)) => quote! { &[#s] },
                Some(serde_json::Value::Array(arr)) => {
                    let items: Vec<&str> = arr.iter().filter_map(|x| x.as_str()).collect();
                    quote! { &[#(#items),*] }
                }
                _ => quote! { &[] },
            };
            quote! {
                ReplaceDiskPredicate::MatchingFluids {
                    offset: pumpkin_util::math::vector3::Vector3::new(#offset_x, #offset_y, #offset_z),
                    fluids: #fluids_token,
                }
            }
        }
        "minecraft:unobstructed" => {
            quote! {
                ReplaceDiskPredicate::Unobstructed
            }
        }
        "minecraft:all_of" => {
            let preds: Vec<TokenStream> = v
                .get("predicates")
                .and_then(|arr| arr.as_array())
                .map(|arr| {
                    arr.iter()
                        .map(parse_single_replace_disk_predicate)
                        .collect()
                })
                .unwrap_or_default();
            quote! {
                ReplaceDiskPredicate::AllOf(&[#(#preds),*])
            }
        }
        _ => quote! {
            ReplaceDiskPredicate::AllOf(&[])
        },
    }
}

fn parse_optional_value_effect(val: Option<&serde_json::Value>) -> TokenStream {
    if let Some(v) = val {
        let eff = v.get("effect").unwrap_or(v);
        let eff_tokens = parse_value_effect(eff);
        quote! { Some(#eff_tokens) }
    } else {
        quote! { None }
    }
}

/// Generates the `TokenStream` for the `Enchantment` struct, `AttributeModifierSlot` enum,
/// data-driven effect types, and `from_name`/`from_id` lookup methods.
pub fn build() -> TokenStream {
    let dir = std::path::Path::new("../../assets/datapacks/26_2/data/minecraft/enchantment");
    let mut enchantments: BTreeMap<String, Enchantment> = BTreeMap::new();
    let mut entries: Vec<_> = fs::read_dir(dir)
        .expect("Missing enchantment directory")
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
        .collect();
    entries.sort_by_key(|e| e.path());

    for (i, entry) in entries.iter().enumerate() {
        let stem = entry
            .path()
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let key = format!("minecraft:{stem}");
        let content = fs::read_to_string(entry.path()).expect("Failed to read enchantment file");
        let mut enc: Enchantment =
            serde_json::from_str(&content).expect("Failed to parse enchantment JSON");
        enc.id = i as u8;
        enchantments.insert(key, enc);
    }

    let mut variants = TokenStream::new();
    let mut all_variants = TokenStream::new();
    let mut name_to_type = TokenStream::new();
    let mut id_to_type = TokenStream::new();

    for (name, enchantment) in enchantments {
        let id = enchantment.id;
        let raw_name = name.strip_prefix("minecraft:").unwrap();
        let format_name = format_ident!("{}", raw_name.to_shouty_snake_case());
        all_variants.extend(quote! { &Self::#format_name, });
        let anvil_cost = enchantment.anvil_cost;
        let supported_items = format_ident!(
            "{}",
            enchantment
                .supported_items
                .strip_prefix("#")
                .unwrap()
                .replace([':', '/'], "_")
                .to_uppercase()
        );
        let max_level = enchantment.max_level;
        let weight = enchantment.weight;
        let min_cost_base = enchantment.min_cost.base;
        let min_cost_per_level = enchantment.min_cost.per_level_above_first;
        let max_cost_base = enchantment.max_cost.base;
        let max_cost_per_level = enchantment.max_cost.per_level_above_first;

        let slots = enchantment.slots;
        let slots = slots.iter().map(AttributeModifierSlot::to_tokens);
        let Translate {
            translate,
            bedrock_translate: _,
            with: _,
        } = &*enchantment.description.0.content
        else {
            panic!()
        };
        let translate = translate.to_string();

        let effects_map = enchantment.effects;
        let projectile_spawned =
            parse_conditional_entity_effects(effects_map.get("minecraft:projectile_spawned"));
        let post_attack =
            parse_targeted_conditional_entity_effects(effects_map.get("minecraft:post_attack"));
        let projectile_count =
            parse_conditional_value_effects(effects_map.get("minecraft:projectile_count"));
        let projectile_spread =
            parse_conditional_value_effects(effects_map.get("minecraft:projectile_spread"));
        let projectile_piercing =
            parse_conditional_value_effects(effects_map.get("minecraft:projectile_piercing"));
        let ammo_use = parse_conditional_value_effects(effects_map.get("minecraft:ammo_use"));
        let damage = parse_conditional_value_effects(effects_map.get("minecraft:damage"));
        let knockback = parse_conditional_value_effects(effects_map.get("minecraft:knockback"));
        let armor_effectiveness =
            parse_conditional_value_effects(effects_map.get("minecraft:armor_effectiveness"));
        let damage_protection =
            parse_conditional_value_effects(effects_map.get("minecraft:damage_protection"));
        let hit_block = parse_conditional_entity_effects(effects_map.get("minecraft:hit_block"));
        let item_damage = parse_conditional_value_effects(effects_map.get("minecraft:item_damage"));
        let equipment_drops =
            parse_targeted_conditional_value_effects(effects_map.get("minecraft:equipment_drops"));
        let fishing_time_reduction =
            parse_conditional_value_effects(effects_map.get("minecraft:fishing_time_reduction"));
        let fishing_luck_bonus =
            parse_conditional_value_effects(effects_map.get("minecraft:fishing_luck_bonus"));
        let block_experience =
            parse_conditional_value_effects(effects_map.get("minecraft:block_experience"));
        let mob_experience =
            parse_conditional_value_effects(effects_map.get("minecraft:mob_experience"));
        let repair_with_xp =
            parse_conditional_value_effects(effects_map.get("minecraft:repair_with_xp"));
        let smash_damage_per_fallen_block = parse_conditional_value_effects(
            effects_map.get("minecraft:smash_damage_per_fallen_block"),
        );
        let trident_return_acceleration = parse_conditional_value_effects(
            effects_map.get("minecraft:trident_return_acceleration"),
        );
        let trident_spin_attack_strength =
            parse_optional_value_effect(effects_map.get("minecraft:trident_spin_attack_strength"));
        let crossbow_charge_time =
            parse_optional_value_effect(effects_map.get("minecraft:crossbow_charge_time"));
        let location_changed =
            parse_conditional_entity_effects(effects_map.get("minecraft:location_changed"));
        let prevent_armor_change = effects_map.contains_key("minecraft:prevent_armor_change");
        let prevent_equipment_drop = effects_map.contains_key("minecraft:prevent_equipment_drop");

        let effects_tokens = quote! {
            EnchantmentEffects {
                projectile_spawned: #projectile_spawned,
                post_attack: #post_attack,
                projectile_count: #projectile_count,
                projectile_spread: #projectile_spread,
                projectile_piercing: #projectile_piercing,
                ammo_use: #ammo_use,
                damage: #damage,
                knockback: #knockback,
                armor_effectiveness: #armor_effectiveness,
                damage_protection: #damage_protection,
                hit_block: #hit_block,
                item_damage: #item_damage,
                equipment_drops: #equipment_drops,
                fishing_time_reduction: #fishing_time_reduction,
                fishing_luck_bonus: #fishing_luck_bonus,
                block_experience: #block_experience,
                mob_experience: #mob_experience,
                repair_with_xp: #repair_with_xp,
                smash_damage_per_fallen_block: #smash_damage_per_fallen_block,
                trident_return_acceleration: #trident_return_acceleration,
                trident_spin_attack_strength: #trident_spin_attack_strength,
                crossbow_charge_time: #crossbow_charge_time,
                location_changed: #location_changed,
                prevent_armor_change: #prevent_armor_change,
                prevent_equipment_drop: #prevent_equipment_drop,
            }
        };

        if let Some(exclusive_set) = &enchantment.exclusive_set {
            let exclusive_set = format_ident!(
                "{}",
                exclusive_set
                    .strip_prefix("#")
                    .unwrap()
                    .replace([':', '/'], "_")
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
                    slots: &[#(#slots),*],
                    weight: #weight,
                    min_cost: Cost {
                        base: #min_cost_base,
                        per_level_above_first: #min_cost_per_level,
                    },
                    max_cost: Cost {
                        base: #max_cost_base,
                        per_level_above_first: #max_cost_per_level,
                    },
                    effects: #effects_tokens,
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
                    slots: &[#(#slots),*],
                    weight: #weight,
                    min_cost: Cost {
                        base: #min_cost_base,
                        per_level_above_first: #min_cost_per_level,
                    },
                    max_cost: Cost {
                        base: #max_cost_base,
                        per_level_above_first: #max_cost_per_level,
                    },
                    effects: #effects_tokens,
                };
            }]);
        }

        name_to_type.extend(quote! { #name | #raw_name => Some(&Self::#format_name), });
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

        #[derive(Clone, Debug, PartialEq)]
        pub enum LevelBasedValue {
            Constant(f32),
            Linear {
                base: f32,
                per_level_above_first: f32,
            },
            Clamped {
                value: &'static Self,
                min: f32,
                max: f32,
            },
            Fraction {
                numerator: &'static Self,
                denominator: &'static Self,
            },
            LevelsSquared {
                added: f32,
            },
            Lookup {
                values: &'static [f32],
                fallback: &'static Self,
            },
        }

        impl LevelBasedValue {
            #[must_use]
            pub fn calculate(&self, level: i32) -> f32 {
                match self {
                    Self::Constant(val) => *val,
                    Self::Linear {
                        base,
                        per_level_above_first,
                    } => base + (level.max(1) - 1) as f32 * per_level_above_first,
                    Self::Clamped { value, min, max } => value.calculate(level).clamp(*min, *max),
                    Self::Fraction {
                        numerator,
                        denominator,
                    } => {
                        let denom = denominator.calculate(level);
                        if denom == 0.0 {
                            0.0
                        } else {
                            numerator.calculate(level) / denom
                        }
                    }
                    Self::LevelsSquared { added } => ((level * level) as f32) + added,
                    Self::Lookup { values, fallback } => {
                        let idx = (level - 1) as usize;
                        values
                            .get(idx)
                            .copied()
                            .unwrap_or_else(|| fallback.calculate(level))
                    }
                }
            }
        }

        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum EnchantmentTarget {
            Attacker,
            DamagingEntity,
            Victim,
        }

        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct TargetedConditionalEffect<T> {
            pub enchanted: Option<EnchantmentTarget>,
            pub affected: Option<EnchantmentTarget>,
            pub effect: T,
        }

        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct ConditionalEffect<T> {
            pub effect: T,
        }

        #[derive(Clone, Debug, PartialEq)]
        pub enum ReplaceDiskPredicate {
            MatchingBlockTag {
                offset: pumpkin_util::math::vector3::Vector3<i32>,
                tag: &'static crate::tag::Tag,
            },
            MatchingBlocks {
                offset: pumpkin_util::math::vector3::Vector3<i32>,
                blocks: &'static [&'static str],
            },
            MatchingFluids {
                offset: pumpkin_util::math::vector3::Vector3<i32>,
                fluids: &'static [&'static str],
            },
            Unobstructed,
            AllOf(&'static [ReplaceDiskPredicate]),
        }

        #[derive(Clone, Debug, PartialEq)]
        pub enum EnchantmentEntityEffect {
            Ignite {
                duration: LevelBasedValue,
            },
            DamageEntity {
                min_damage: LevelBasedValue,
                max_damage: LevelBasedValue,
                damage_type: Option<&'static crate::damage::DamageType>,
            },
            ChangeItemDamage {
                amount: LevelBasedValue,
            },
            PlaySound {
                sound: &'static str,
            },
            ReplaceBlock {
                offset_x: i32,
                offset_y: i32,
                offset_z: i32,
                trigger_game_event: Option<crate::game_event::GameEvent>,
            },
            SetBlockProperties {
                properties: &'static [(&'static str, &'static str)],
                offset_x: i32,
                offset_y: i32,
                offset_z: i32,
                trigger_game_event: Option<crate::game_event::GameEvent>,
            },
            ReplaceDisk {
                radius: LevelBasedValue,
                height: LevelBasedValue,
                offset_x: i32,
                offset_y: i32,
                offset_z: i32,
                predicate: Option<ReplaceDiskPredicate>,
                block_state: &'static crate::block_state::BlockState,
                trigger_game_event: Option<crate::game_event::GameEvent>,
            },
            SummonEntity {
                entity_types: &'static [&'static crate::entity::EntityType],
                join_team: bool,
            },
            SpawnParticles {
                particle: crate::particle::Particle,
                horizontal_position: PositionSource,
                vertical_position: PositionSource,
                horizontal_velocity: VelocitySource,
                vertical_velocity: VelocitySource,
                speed: pumpkin_util::math::float_provider::FloatProvider,
            },
            RunFunction {
                function: &'static str,
            },
            ApplyExhaustion {
                amount: LevelBasedValue,
            },
            ApplyImpulse {
                direction: pumpkin_util::math::vector3::Vector3<f64>,
                coordinate_scale: pumpkin_util::math::vector3::Vector3<f64>,
                magnitude: LevelBasedValue,
            },
            ApplyMobEffect {
                to_apply: &'static [&'static crate::effect::StatusEffect],
                min_duration: LevelBasedValue,
                max_duration: LevelBasedValue,
                min_amplifier: LevelBasedValue,
                max_amplifier: LevelBasedValue,
            },
            Explode {
                attribute_to_user: bool,
                damage_type: Option<&'static crate::damage::DamageType>,
                knockback_multiplier: Option<LevelBasedValue>,
                immune_blocks: Option<&'static str>,
                offset_x: f64,
                offset_y: f64,
                offset_z: f64,
                radius: LevelBasedValue,
                create_fire: bool,
                block_interaction: &'static str,
                small_particle: Option<crate::particle::Particle>,
                large_particle: Option<crate::particle::Particle>,
                sound: Option<crate::sound::Sound>,
            },
            AllOf(&'static [EnchantmentEntityEffect]),
            Other,
        }

        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
        pub enum PositionSourceType {
            #[default]
            EntityPosition,
            InBoundingBox,
        }

        impl PositionSourceType {
            #[must_use]
            pub fn get_coordinate(
                self,
                position: f64,
                center: f64,
                bounding_box_span: f32,
                random: &mut impl pumpkin_util::random::RandomImpl,
            ) -> f64 {
                match self {
                    Self::EntityPosition => position,
                    Self::InBoundingBox => {
                        let random_offset = f64::from(random.next_f32()) - 0.5;
                        center + random_offset * f64::from(bounding_box_span)
                    }
                }
            }
        }

        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct PositionSource {
            pub source_type: PositionSourceType,
            pub offset: f32,
            pub scale: f32,
        }

        impl PositionSource {
            #[must_use]
            pub const fn new(source_type: PositionSourceType, offset: f32, scale: f32) -> Self {
                Self {
                    source_type,
                    offset,
                    scale,
                }
            }

            #[must_use]
            pub const fn offset_from_entity_position(offset: f32) -> Self {
                Self {
                    source_type: PositionSourceType::EntityPosition,
                    offset,
                    scale: 1.0,
                }
            }

            #[must_use]
            pub const fn in_bounding_box() -> Self {
                Self {
                    source_type: PositionSourceType::InBoundingBox,
                    offset: 0.0,
                    scale: 1.0,
                }
            }

            #[must_use]
            pub fn get_coordinate(
                &self,
                position: f64,
                center: f64,
                bounding_box_span: f32,
                random: &mut impl pumpkin_util::random::RandomImpl,
            ) -> f64 {
                self.source_type.get_coordinate(
                    position,
                    center,
                    bounding_box_span * self.scale,
                    random,
                ) + f64::from(self.offset)
            }
        }

        #[derive(Clone, Debug, PartialEq)]
        pub struct VelocitySource {
            pub movement_scale: f32,
            pub base: pumpkin_util::math::float_provider::FloatProvider,
        }

        impl VelocitySource {
            #[must_use]
            pub const fn new(
                movement_scale: f32,
                base: pumpkin_util::math::float_provider::FloatProvider,
            ) -> Self {
                Self {
                    movement_scale,
                    base,
                }
            }

            #[must_use]
            pub const fn movement_scaled(scale: f32) -> Self {
                Self {
                    movement_scale: scale,
                    base: pumpkin_util::math::float_provider::FloatProvider::Constant(0.0),
                }
            }

            #[must_use]
            pub const fn fixed_velocity(
                provider: pumpkin_util::math::float_provider::FloatProvider,
            ) -> Self {
                Self {
                    movement_scale: 0.0,
                    base: provider,
                }
            }

            pub fn get_velocity(
                &self,
                movement: f64,
                random: &mut impl pumpkin_util::random::RandomImpl,
            ) -> f64 {
                f64::from(self.movement_scale).mul_add(movement, f64::from(self.base.get(random)))
            }
        }

        #[derive(Clone, Debug, PartialEq)]
        pub enum EnchantmentValueEffect {
            Add(LevelBasedValue),
            Multiply(LevelBasedValue),
            Set(LevelBasedValue),
            RemoveBinomial(LevelBasedValue),
            Other,
        }

        impl EnchantmentValueEffect {
            #[must_use]
            pub fn process(&self, level: i32, current_value: f32) -> f32 {
                match self {
                    Self::Add(val) => current_value + val.calculate(level),
                    Self::Multiply(val) => current_value * val.calculate(level),
                    Self::Set(val) => val.calculate(level),
                    Self::RemoveBinomial(val) => {
                        let prob = val.calculate(level);
                        if rand::random::<f32>() < prob {
                            0.0
                        } else {
                            current_value
                        }
                    }
                    Self::Other => current_value,
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct EnchantmentEffects {
            pub projectile_spawned: &'static [ConditionalEffect<EnchantmentEntityEffect>],
            pub post_attack: &'static [TargetedConditionalEffect<EnchantmentEntityEffect>],
            pub projectile_count: &'static [ConditionalEffect<EnchantmentValueEffect>],
            pub projectile_spread: &'static [ConditionalEffect<EnchantmentValueEffect>],
            pub projectile_piercing: &'static [ConditionalEffect<EnchantmentValueEffect>],
            pub ammo_use: &'static [ConditionalEffect<EnchantmentValueEffect>],
            pub damage: &'static [ConditionalEffect<EnchantmentValueEffect>],
            pub knockback: &'static [ConditionalEffect<EnchantmentValueEffect>],
            pub armor_effectiveness: &'static [ConditionalEffect<EnchantmentValueEffect>],
            pub damage_protection: &'static [ConditionalEffect<EnchantmentValueEffect>],
            pub hit_block: &'static [ConditionalEffect<EnchantmentEntityEffect>],
            pub item_damage: &'static [ConditionalEffect<EnchantmentValueEffect>],
            pub equipment_drops: &'static [TargetedConditionalEffect<EnchantmentValueEffect>],
            pub fishing_time_reduction: &'static [ConditionalEffect<EnchantmentValueEffect>],
            pub fishing_luck_bonus: &'static [ConditionalEffect<EnchantmentValueEffect>],
            pub block_experience: &'static [ConditionalEffect<EnchantmentValueEffect>],
            pub mob_experience: &'static [ConditionalEffect<EnchantmentValueEffect>],
            pub repair_with_xp: &'static [ConditionalEffect<EnchantmentValueEffect>],
            pub smash_damage_per_fallen_block: &'static [ConditionalEffect<EnchantmentValueEffect>],
            pub trident_return_acceleration: &'static [ConditionalEffect<EnchantmentValueEffect>],
            pub trident_spin_attack_strength: Option<EnchantmentValueEffect>,
            pub crossbow_charge_time: Option<EnchantmentValueEffect>,
            pub location_changed: &'static [ConditionalEffect<EnchantmentEntityEffect>],
            pub prevent_armor_change: bool,
            pub prevent_equipment_drop: bool,
        }

        pub struct Enchantment {
            pub id: u8,
            pub name: &'static str,
            pub registry_key: &'static str,
            pub description: &'static str, // TODO use TextComponent
            pub anvil_cost: u32,
            pub supported_items: &'static Tag,
            pub exclusive_set: Option<&'static Tag>,
            pub max_level: i32,
            pub slots: &'static [AttributeModifierSlot],
            pub weight: i32,
            pub min_cost: Cost,
            pub max_cost: Cost,
            pub effects: EnchantmentEffects,
        }

        #[derive(Clone, Copy, Debug)]
        pub struct Cost {
            pub base: i32,
            pub per_level_above_first: i32,
        }

        impl Cost {
            pub fn calculate(&self, level: i32) -> i32 {
                self.base + self.per_level_above_first * (level - 1)
            }
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
            pub const ALL: &'static [&'static Self] = &[#all_variants];

            pub fn all() -> Iter<'static, &'static Self> {
                Self::ALL.iter()
            }

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
            #[allow(deprecated)]
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

            pub fn modify_damage_protection(&self, level: i32, protection: &mut f32) {
                for effect in self.effects.damage_protection {
                    *protection = effect.effect.process(level, *protection);
                }
            }

            pub fn modify_damage(&self, level: i32, amount: &mut f64) {
                for effect in self.effects.damage {
                    *amount = f64::from(effect.effect.process(level, *amount as f32));
                }
            }

            pub fn modify_fall_based_damage(&self, level: i32, amount: &mut f64) {
                for effect in self.effects.smash_damage_per_fallen_block {
                    *amount = f64::from(effect.effect.process(level, *amount as f32));
                }
            }

            pub fn modify_knockback(&self, level: i32, amount: &mut f32) {
                for effect in self.effects.knockback {
                    *amount = effect.effect.process(level, *amount);
                }
            }

            pub fn modify_armor_effectiveness(&self, level: i32, amount: &mut f32) {
                for effect in self.effects.armor_effectiveness {
                    *amount = effect.effect.process(level, *amount);
                }
            }

            pub fn modify_durability_change(&self, level: i32, change: &mut f32) {
                for effect in self.effects.item_damage {
                    *change = effect.effect.process(level, *change);
                }
            }

            pub fn modify_ammo_count(&self, level: i32, change: &mut f32) {
                for effect in self.effects.ammo_use {
                    *change = effect.effect.process(level, *change);
                }
            }

            pub fn modify_piercing_count(&self, level: i32, count: &mut f32) {
                for effect in self.effects.projectile_piercing {
                    *count = effect.effect.process(level, *count);
                }
            }

            pub fn modify_block_experience(&self, level: i32, count: &mut f32) {
                for effect in self.effects.block_experience {
                    *count = effect.effect.process(level, *count);
                }
            }

            pub fn modify_mob_experience(&self, level: i32, experience: &mut f32) {
                for effect in self.effects.mob_experience {
                    *experience = effect.effect.process(level, *experience);
                }
            }

            pub fn modify_durability_to_repair_from_xp(&self, level: i32, change: &mut f32) {
                for effect in self.effects.repair_with_xp {
                    *change = effect.effect.process(level, *change);
                }
            }

            pub fn modify_trident_return_to_owner_acceleration(&self, level: i32, count: &mut f32) {
                for effect in self.effects.trident_return_acceleration {
                    *count = effect.effect.process(level, *count);
                }
            }

            pub fn modify_trident_spin_attack_strength(&self, level: i32, strength: &mut f32) {
                if let Some(effect) = &self.effects.trident_spin_attack_strength {
                    *strength = effect.process(level, *strength);
                }
            }

            pub fn modify_fishing_time_reduction(&self, level: i32, time_reduction: &mut f32) {
                for effect in self.effects.fishing_time_reduction {
                    *time_reduction = effect.effect.process(level, *time_reduction);
                }
            }

            pub fn modify_fishing_luck_bonus(&self, level: i32, luck: &mut f32) {
                for effect in self.effects.fishing_luck_bonus {
                    *luck = effect.effect.process(level, *luck);
                }
            }

            pub fn modify_projectile_count(&self, level: i32, count: &mut f32) {
                for effect in self.effects.projectile_count {
                    *count = effect.effect.process(level, *count);
                }
            }

            pub fn modify_projectile_spread(&self, level: i32, angle: &mut f32) {
                for effect in self.effects.projectile_spread {
                    *angle = effect.effect.process(level, *angle);
                }
            }

            pub fn modify_crossbow_charge_time(&self, level: i32, time: &mut f32) {
                if let Some(effect) = &self.effects.crossbow_charge_time {
                    *time = effect.process(level, *time);
                }
            }

            pub fn get_projectile_spawned_effects(&self) -> &'static [ConditionalEffect<EnchantmentEntityEffect>] {
                self.effects.projectile_spawned
            }

            pub fn get_location_changed_effects(&self) -> &'static [ConditionalEffect<EnchantmentEntityEffect>] {
                self.effects.location_changed
            }
        }
    }
}
