pub mod evaluate;
pub mod types;

use std::collections::HashMap;

use crate::Identifier;
use crate::resource::ResourceManager;

pub use types::*;

/// Load loot tables from datapacks.
pub fn load_loot_tables(
    manager: &dyn ResourceManager,
) -> Result<HashMap<Identifier, LootTable>, crate::DatapackError> {
    let mut tables = HashMap::new();

    for ns in manager.get_namespaces() {
        let paths =
            crate::resource::list_resources_multi(manager, &ns, &["loot_table", "loot_tables"]);
        for path in &paths {
            if !std::path::Path::new(path)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
            {
                continue;
            }
            let Some(data) = manager.get_resource(&ns, path) else {
                continue;
            };
            let raw: serde_json::Value = serde_json::from_slice(&data)?;

            let table_name = path
                .strip_prefix("loot_table/")
                .or_else(|| path.strip_prefix("loot_tables/"))
                .and_then(|p| p.strip_suffix(".json"))
                .unwrap_or(path.as_str());
            let id = Identifier::new(ns.clone(), table_name.to_string())?;

            let loot_type = raw
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("minecraft:chest")
                .to_string();

            let random_sequence = raw
                .get("random_sequence")
                .and_then(|v| v.as_str())
                .map(String::from);

            let pools = parse_pools(raw.get("pools"));
            tables.insert(
                id.clone(),
                LootTable {
                    id,
                    loot_type,
                    pools,
                    random_sequence,
                },
            );
        }
    }

    Ok(tables)
}

fn parse_pools(val: Option<&serde_json::Value>) -> Vec<LootPool> {
    let Some(val) = val else {
        return Vec::new();
    };
    let serde_json::Value::Array(arr) = val else {
        return Vec::new();
    };
    arr.iter().map(parse_pool).collect()
}

fn parse_pool(v: &serde_json::Value) -> LootPool {
    LootPool {
        rolls: parse_number_provider(v.get("rolls")),
        bonus_rolls: parse_number_provider(v.get("bonus_rolls")),
        conditions: parse_conditions(v.get("conditions")),
        entries: parse_entries(v.get("entries")),
        functions: parse_functions(v.get("functions")),
    }
}

fn parse_entries(val: Option<&serde_json::Value>) -> Vec<LootEntry> {
    let Some(val) = val else {
        return Vec::new();
    };
    let serde_json::Value::Array(arr) = val else {
        return Vec::new();
    };
    arr.iter().map(parse_entry).collect()
}

fn parse_entry(v: &serde_json::Value) -> LootEntry {
    let entry_type = v
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("minecraft:item");
    let content = parse_entry_type(entry_type, v);
    LootEntry {
        weight: v
            .get("weight")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(1) as i32,
        quality: v
            .get("quality")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0) as i32,
        content,
        conditions: parse_conditions(v.get("conditions")),
        functions: parse_functions(v.get("functions")),
    }
}

fn parse_entry_type(entry_type: &str, v: &serde_json::Value) -> LootEntryType {
    match entry_type {
        "minecraft:item" | "item" => LootEntryType::Item(
            v.get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("minecraft:air")
                .to_string(),
        ),
        "minecraft:loot_table" | "loot_table" => LootEntryType::LootTable(
            v.get("value")
                .and_then(|n| n.as_str())
                .unwrap_or("minecraft:empty")
                .to_string(),
        ),
        "minecraft:tag" | "tag" => LootEntryType::Tag {
            name: v
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("")
                .to_string(),
            expand: v
                .get("expand")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
        },
        "minecraft:alternatives" | "alternatives" => LootEntryType::Alternatives(parse_children(v)),
        "minecraft:sequence" | "sequence" => LootEntryType::Sequence(parse_children(v)),
        "minecraft:group" | "group" => LootEntryType::Group(parse_children(v)),
        "minecraft:empty" | "empty" | "minecraft:dynamic" | "dynamic" => LootEntryType::Empty,
        _ => {
            tracing::warn!("Unknown loot entry type: {entry_type}, treating as empty");
            LootEntryType::Empty
        }
    }
}

fn parse_children(v: &serde_json::Value) -> Vec<LootEntry> {
    v.get("children")
        .and_then(serde_json::Value::as_array)
        .map(|arr| arr.iter().map(parse_entry).collect())
        .unwrap_or_default()
}

fn parse_conditions(val: Option<&serde_json::Value>) -> Vec<LootCondition> {
    let Some(val) = val else {
        return Vec::new();
    };
    let serde_json::Value::Array(arr) = val else {
        return Vec::new();
    };
    arr.iter().filter_map(parse_condition).collect()
}

#[allow(clippy::too_many_lines)]
fn parse_condition(v: &serde_json::Value) -> Option<LootCondition> {
    let condition = v.get("condition")?.as_str()?;
    Some(match condition {
        "minecraft:survives_explosion" | "survives_explosion" => LootCondition::SurvivesExplosion,
        "minecraft:killed_by_player" | "killed_by_player" => LootCondition::KilledByPlayer,
        "minecraft:random_chance" | "random_chance" => LootCondition::RandomChance(
            v.get("chance")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(1.0) as f32,
        ),
        "minecraft:random_chance_with_enchanted_bonus" | "random_chance_with_enchanted_bonus" => {
            LootCondition::RandomChanceWithLooting {
                enchantment: v
                    .get("enchantment")
                    .and_then(|e| e.as_str())
                    .unwrap_or("")
                    .to_string(),
                unenchanted_chance: v
                    .get("unenchanted_chance")
                    .and_then(serde_json::Value::as_f64)
                    .unwrap_or(0.0) as f32,
                enchanted_chance: parse_number_provider(v.get("enchanted_chance")),
            }
        }
        "minecraft:match_tool" | "match_tool" => {
            let items = v
                .get("predicate")
                .and_then(|p| p.get("items"))
                .and_then(|i| i.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|i| i.as_str().map(String::from))
                        .collect()
                });
            LootCondition::MatchTool {
                items,
                require_silk_touch: false,
            }
        }
        "minecraft:entity_properties" | "entity_properties" => LootCondition::EntityProperties {
            entity: v
                .get("entity")
                .and_then(|e| e.as_str())
                .unwrap_or("this")
                .to_string(),
            predicate: v
                .get("predicate")
                .cloned()
                .unwrap_or(serde_json::Value::Null),
        },
        "minecraft:damage_source_properties" | "damage_source_properties" => {
            LootCondition::DamageSourceProperties(
                v.get("predicate")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null),
            )
        }
        "minecraft:location_check" | "location_check" => LootCondition::LocationCheck(
            v.get("predicate")
                .cloned()
                .unwrap_or(serde_json::Value::Null),
        ),
        "minecraft:weather_check" | "weather_check" => LootCondition::WeatherCheck {
            raining: v.get("raining").and_then(serde_json::Value::as_bool),
            thundering: v.get("thundering").and_then(serde_json::Value::as_bool),
        },
        "minecraft:table_bonus" | "table_bonus" => LootCondition::TableBonus {
            enchantment: v
                .get("enchantment")
                .and_then(|e| e.as_str())
                .unwrap_or("")
                .to_string(),
            chances: v
                .get("chances")
                .and_then(|c| c.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|c| c.as_f64().map(|f| f as f32))
                        .collect()
                })
                .unwrap_or_default(),
        },
        "minecraft:entity_scores" | "entity_scores" => LootCondition::EntityScores {
            entity: v
                .get("entity")
                .and_then(|e| e.as_str())
                .unwrap_or("this")
                .to_string(),
        },
        "minecraft:time_check" | "time_check" => LootCondition::TimeCheck {
            value: v.get("value").map(|v| parse_number_provider(Some(v))),
            period: v.get("period").and_then(serde_json::Value::as_i64),
        },
        "minecraft:value_check" | "value_check" => {
            let value = parse_number_provider(v.get("value"));
            let range_min = v
                .get("range")
                .and_then(|r| r.get("min"))
                .and_then(serde_json::Value::as_f64)
                .map(|f| f as f32);
            let range_max = v
                .get("range")
                .and_then(|r| r.get("max"))
                .and_then(serde_json::Value::as_f64)
                .map(|f| f as f32);
            LootCondition::ValueCheck {
                value,
                range: (range_min, range_max),
            }
        }
        "minecraft:reference" | "reference" => LootCondition::Reference(
            v.get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("")
                .to_string(),
        ),
        "minecraft:enchantment_active_check" | "enchantment_active_check" => {
            LootCondition::EnchantmentActiveCheck(
                v.get("active")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(true),
            )
        }
        "minecraft:block_state_property" | "block_state_property" => {
            LootCondition::BlockStateProperty(
                v.get("block").cloned().unwrap_or(serde_json::Value::Null),
            )
        }
        "minecraft:inverted" | "inverted" => LootCondition::Inverted(Box::new(
            v.get("term")
                .and_then(parse_condition)
                .unwrap_or(LootCondition::SurvivesExplosion),
        )),
        "minecraft:alternative" | "alternative" | "minecraft:any_of" | "any_of" => {
            LootCondition::AnyOf(
                v.get("terms")
                    .and_then(|t| t.as_array())
                    .map(|arr| arr.iter().filter_map(parse_condition).collect())
                    .unwrap_or_default(),
            )
        }
        "minecraft:all_of" | "all_of" => LootCondition::AllOf(
            v.get("terms")
                .and_then(|t| t.as_array())
                .map(|arr| arr.iter().filter_map(parse_condition).collect())
                .unwrap_or_default(),
        ),
        other => {
            tracing::warn!("Unknown loot condition type: {other}");
            return None;
        }
    })
}

fn parse_functions(val: Option<&serde_json::Value>) -> Vec<LootFunction> {
    let Some(val) = val else {
        return Vec::new();
    };
    let serde_json::Value::Array(arr) = val else {
        return Vec::new();
    };
    arr.iter().filter_map(parse_function).collect()
}

#[allow(clippy::too_many_lines)]
fn parse_function(v: &serde_json::Value) -> Option<LootFunction> {
    let function = v.get("function")?.as_str()?;
    let conditions = parse_conditions(v.get("conditions"));
    let content = match function {
        "minecraft:set_count" | "set_count" => LootFunctionType::SetCount {
            count: parse_number_provider(v.get("count")),
            add: v
                .get("add")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
        },
        "minecraft:set_damage" | "set_damage" => LootFunctionType::SetDamage {
            damage: parse_number_provider(v.get("damage")),
            add: v
                .get("add")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
        },
        "minecraft:set_components" | "set_components" => LootFunctionType::SetComponents(
            v.get("components")
                .cloned()
                .unwrap_or(serde_json::Value::Null),
        ),
        "minecraft:copy_components" | "copy_components" => LootFunctionType::CopyComponents {
            source: v
                .get("source")
                .and_then(|s| s.as_str())
                .unwrap_or("block_entity")
                .to_string(),
        },
        "minecraft:furnace_smelt" | "furnace_smelt" => LootFunctionType::FurnaceSmelt,
        "minecraft:enchanted_count_increase" | "enchanted_count_increase" => {
            LootFunctionType::EnchantedCountIncrease {
                enchantment: v
                    .get("enchantment")
                    .and_then(|e| e.as_str())
                    .unwrap_or("")
                    .to_string(),
                count: parse_number_provider(v.get("count")),
                limit: v
                    .get("limit")
                    .and_then(serde_json::Value::as_f64)
                    .map(|f| f as f32),
            }
        }
        "minecraft:apply_bonus" | "apply_bonus" => LootFunctionType::ApplyBonus {
            enchantment: v
                .get("enchantment")
                .and_then(|e| e.as_str())
                .unwrap_or("")
                .to_string(),
            formula: v
                .get("formula")
                .and_then(|f| f.as_str())
                .unwrap_or("")
                .to_string(),
            parameters: v.get("parameters").cloned(),
        },
        "minecraft:limit_count" | "limit_count" => LootFunctionType::LimitCount {
            min: v
                .get("limit")
                .and_then(|l| l.get("min"))
                .and_then(serde_json::Value::as_f64)
                .map(|f| f as f32),
            max: v
                .get("limit")
                .and_then(|l| l.get("max"))
                .and_then(serde_json::Value::as_f64)
                .map(|f| f as f32),
        },
        "minecraft:explosion_decay" | "explosion_decay" => LootFunctionType::ExplosionDecay,
        "minecraft:set_potion" | "set_potion" => LootFunctionType::SetPotion {
            id: v
                .get("id")
                .and_then(|i| i.as_str())
                .unwrap_or("")
                .to_string(),
        },
        "minecraft:set_ominous_bottle_amplifier" | "set_ominous_bottle_amplifier" => {
            LootFunctionType::SetOminousBottleAmplifier
        }
        "minecraft:copy_state" | "copy_state" => LootFunctionType::CopyState(
            v.get("block")
                .or_else(|| v.get("properties"))
                .cloned()
                .unwrap_or(serde_json::Value::Null),
        ),
        "minecraft:enchant_randomly" | "enchant_randomly" => {
            LootFunctionType::EnchantRandomly(v.clone())
        }
        "minecraft:enchant_with_levels" | "enchant_with_levels" => {
            LootFunctionType::EnchantWithLevels(v.clone())
        }
        "minecraft:set_stew_effect" | "set_stew_effect" => {
            LootFunctionType::SetStewEffect(v.clone())
        }
        "minecraft:set_instrument" | "set_instrument" => LootFunctionType::SetInstrument(v.clone()),
        "minecraft:exploration_map" | "exploration_map" => {
            LootFunctionType::ExplorationMap(v.clone())
        }
        "minecraft:set_name" | "set_name" => LootFunctionType::SetName(v.clone()),
        "minecraft:set_enchantments" | "set_enchantments" => {
            LootFunctionType::SetEnchantments(v.clone())
        }
        "minecraft:copy_custom_data" | "copy_custom_data" => {
            LootFunctionType::CopyCustomData(v.clone())
        }
        "minecraft:set_custom_data" | "set_custom_data" => {
            LootFunctionType::SetCustomData(v.clone())
        }
        "minecraft:filtered" | "filtered" => LootFunctionType::Filtered(v.clone()),
        other => {
            tracing::warn!("Unknown loot function type: {other}");
            return None;
        }
    };
    Some(LootFunction {
        content,
        conditions,
    })
}

fn parse_number_provider(val: Option<&serde_json::Value>) -> LootNumberProvider {
    let Some(val) = val else {
        return LootNumberProvider::Constant(0.0);
    };
    match val {
        serde_json::Value::Number(n) => {
            LootNumberProvider::Constant(n.as_f64().unwrap_or(0.0) as f32)
        }
        serde_json::Value::Object(_) => {
            let min = val
                .get("min")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0) as f32;
            let max = val
                .get("max")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0) as f32;
            let n = val
                .get("n")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0) as i32;
            let p = val
                .get("p")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0) as f32;
            let r#type = val.get("type").and_then(|t| t.as_str());
            match r#type {
                Some("minecraft:uniform" | "uniform") => LootNumberProvider::Uniform { min, max },
                Some("minecraft:binomial" | "binomial") => LootNumberProvider::Binomial { n, p },
                _ => {
                    // Object without type field - try min/max as uniform
                    if val.get("min").is_some() || val.get("max").is_some() {
                        LootNumberProvider::Uniform { min, max }
                    } else {
                        LootNumberProvider::Constant(
                            val.get("base")
                                .and_then(serde_json::Value::as_f64)
                                .unwrap_or(0.0) as f32,
                        )
                    }
                }
            }
        }
        _ => LootNumberProvider::Constant(0.0),
    }
}
