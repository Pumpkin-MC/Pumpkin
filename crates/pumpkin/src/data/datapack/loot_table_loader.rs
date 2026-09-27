use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use pumpkin_util::loot_table::{
    DynamicLootCondition, DynamicLootEntry, DynamicLootPool, DynamicLootTable, LootBonusFormula,
};
use serde_json::Value;

/// Parses an entire loot table JSON string into a [`DynamicLootTable`].
#[must_use]
pub fn parse_loot_table(json_content: &str) -> Option<DynamicLootTable> {
    let root: Value = serde_json::from_str(json_content).ok()?;
    let pools_val = root.get("pools").and_then(Value::as_array)?;

    let pools = pools_val.iter().map(parse_pool).collect();

    Some(DynamicLootTable { pools })
}

fn parse_pool(val: &Value) -> DynamicLootPool {
    let (min_rolls, max_rolls) = val.get("rolls").map_or((1, 1), parse_rolls);

    let mut entries = Vec::new();
    let mut empty_weight = 0;

    if let Some(entries_val) = val.get("entries").and_then(Value::as_array) {
        for entry_val in entries_val {
            parse_entry(entry_val, &mut entries, &mut empty_weight);
        }
    }

    let pool_conditions = val.get("conditions").map_or(Vec::new(), parse_conditions);
    let condition = combine_conditions(pool_conditions);

    DynamicLootPool {
        entries,
        min_rolls,
        max_rolls,
        empty_weight,
        condition,
    }
}

fn parse_rolls(val: &Value) -> (i32, i32) {
    if let Some(n) = val.as_i64() {
        let r = n as i32;
        return (r, r);
    }
    if let Some(f) = val.as_f64() {
        let r = f as i32;
        return (r, r);
    }
    if let Some(obj) = val.as_object() {
        if let (Some(min), Some(max)) = (obj.get("min"), obj.get("max")) {
            let min_r = min
                .as_i64()
                .or_else(|| min.as_f64().map(|f| f as i64))
                .unwrap_or(1) as i32;
            let max_r = max
                .as_i64()
                .or_else(|| max.as_f64().map(|f| f as i64))
                .unwrap_or(min_r as i64) as i32;
            return (min_r, max_r);
        }
        if let Some(value) = obj.get("value") {
            let r = value
                .as_i64()
                .or_else(|| value.as_f64().map(|f| f as i64))
                .unwrap_or(1) as i32;
            return (r, r);
        }
        if let Some(n) = obj.get("n").and_then(Value::as_i64) {
            return (0, n as i32);
        }
    }
    (1, 1)
}

fn parse_entry(val: &Value, pool_entries: &mut Vec<DynamicLootEntry>, empty_weight: &mut i32) {
    let entry_type = val
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("minecraft:item");
    let entry_type = entry_type.strip_prefix("minecraft:").unwrap_or(entry_type);
    let weight = val.get("weight").and_then(Value::as_i64).unwrap_or(1) as i32;

    match entry_type {
        "empty" => {
            *empty_weight += weight;
        }
        "item" | "tag" => {
            if let Some(name) = val.get("name").and_then(Value::as_str) {
                let (min_count, max_count, bonus_formula) =
                    val.get("functions").map_or((1, 1, None), parse_functions);
                let conditions = val.get("conditions").map_or(Vec::new(), parse_conditions);
                let condition = combine_conditions(conditions);

                pool_entries.push(DynamicLootEntry {
                    item: name.to_string(),
                    weight,
                    min_count,
                    max_count,
                    condition,
                    bonus_formula,
                });
            }
        }
        "alternatives" | "group" | "sequence" => {
            if let Some(children) = val.get("children").and_then(Value::as_array) {
                for child in children {
                    parse_entry(child, pool_entries, empty_weight);
                }
            }
        }
        _ => {}
    }
}

fn parse_functions(val: &Value) -> (i32, i32, Option<LootBonusFormula>) {
    let mut min_count = 1;
    let mut max_count = 1;
    let mut bonus_formula = None;

    let Some(functions) = val.as_array() else {
        return (min_count, max_count, bonus_formula);
    };

    for func in functions {
        let func_type = func
            .get("function")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let func_type = func_type.strip_prefix("minecraft:").unwrap_or(func_type);

        match func_type {
            "set_count" => {
                if let Some(count) = func.get("count") {
                    let (min_c, max_c) = parse_rolls(count);
                    min_count = min_c;
                    max_count = max_c;
                }
            }
            "apply_bonus" => {
                if let Some(formula) = func.get("formula").and_then(Value::as_str) {
                    let formula_clean = formula.strip_prefix("minecraft:").unwrap_or(formula);
                    match formula_clean {
                        "ore_drops" => {
                            bonus_formula = Some(LootBonusFormula::OreDrops);
                        }
                        "uniform_bonus_count" => {
                            let multiplier = func
                                .get("parameters")
                                .and_then(|p| p.get("bonusMultiplier"))
                                .and_then(Value::as_i64)
                                .unwrap_or(1) as i32;
                            bonus_formula = Some(LootBonusFormula::UniformBonusCount(multiplier));
                        }
                        "binomial_with_bonus_count" => {
                            let extra = func
                                .get("parameters")
                                .and_then(|p| p.get("extra"))
                                .and_then(Value::as_i64)
                                .unwrap_or(0) as i32;
                            let probability =
                                func.get("parameters")
                                    .and_then(|p| p.get("probability"))
                                    .and_then(Value::as_f64)
                                    .unwrap_or(0.0) as f32;
                            bonus_formula = Some(LootBonusFormula::BinomialWithBonusCount {
                                extra,
                                probability,
                            });
                        }
                        _ => {}
                    }
                }
            }
            "looting_enchant" => {
                let max_bonus = func.get("count").map_or(1, |c| {
                    c.get("max")
                        .and_then(Value::as_i64)
                        .or_else(|| c.as_i64())
                        .unwrap_or(1) as i32
                });
                bonus_formula = Some(LootBonusFormula::UniformBonusCount(max_bonus.max(1)));
            }
            _ => {}
        }
    }

    (min_count, max_count, bonus_formula)
}

fn parse_conditions(val: &Value) -> Vec<DynamicLootCondition> {
    let Some(arr) = val.as_array() else {
        return Vec::new();
    };
    arr.iter().map(parse_condition).collect()
}

fn parse_condition(val: &Value) -> DynamicLootCondition {
    let cond_type = val
        .get("condition")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let cond_type = cond_type.strip_prefix("minecraft:").unwrap_or(cond_type);

    match cond_type {
        "survives_explosion" => DynamicLootCondition::SurvivesExplosion,
        "killed_by_player" => DynamicLootCondition::KilledByPlayer,
        "random_chance" => {
            let chance = val.get("chance").and_then(Value::as_f64).unwrap_or(1.0) as f32;
            DynamicLootCondition::RandomChance { chance }
        }
        "random_chance_with_enchanted_bonus" => {
            let unenchanted = val
                .get("unenchanted_chance")
                .and_then(Value::as_f64)
                .unwrap_or(0.0) as f32;
            let enchanted_base = val
                .get("enchanted_chance")
                .or_else(|| val.get("enchanted_chance_base"))
                .and_then(Value::as_f64)
                .unwrap_or(f64::from(unenchanted)) as f32;
            let per_level = val
                .get("enchanted_chance_per_level_above_first")
                .and_then(Value::as_f64)
                .unwrap_or(0.0) as f32;
            DynamicLootCondition::RandomChanceWithEnchantedBonus {
                unenchanted_chance: unenchanted,
                enchanted_chance_base: enchanted_base,
                enchanted_chance_per_level_above_first: per_level,
            }
        }
        "table_bonus" => {
            let chances: Vec<f32> = val
                .get("chances")
                .and_then(Value::as_array)
                .map(|arr| {
                    arr.iter()
                        .filter_map(|x| x.as_f64().map(|f| f as f32))
                        .collect()
                })
                .unwrap_or_default();
            DynamicLootCondition::TableBonus {
                chances: chances.into_boxed_slice(),
            }
        }
        "inverted" => val.get("term").map_or(DynamicLootCondition::None, |term| {
            DynamicLootCondition::Inverted(Box::new(parse_condition(term)))
        }),
        "any_of" => val
            .get("terms")
            .and_then(Value::as_array)
            .map_or(DynamicLootCondition::None, |terms| {
                DynamicLootCondition::AnyOf(terms.iter().map(parse_condition).collect())
            }),
        "all_of" => val
            .get("terms")
            .and_then(Value::as_array)
            .map_or(DynamicLootCondition::None, |terms| {
                DynamicLootCondition::AllOf(terms.iter().map(parse_condition).collect())
            }),
        "match_tool" => {
            let pred_str = val
                .get("predicate")
                .map_or_else(String::new, Value::to_string);
            let has_silk = pred_str.contains("silk_touch");
            let has_shears = pred_str.contains("shears");
            if has_silk && has_shears {
                DynamicLootCondition::SilkTouchOrShears
            } else if has_silk {
                DynamicLootCondition::SilkTouch
            } else if has_shears {
                DynamicLootCondition::Shears
            } else {
                DynamicLootCondition::None
            }
        }
        "entity_properties" => {
            let on_fire = val
                .get("predicate")
                .and_then(|p| p.get("flags"))
                .and_then(|f| f.get("is_on_fire"))
                .and_then(Value::as_bool)
                .unwrap_or(false);
            if on_fire {
                DynamicLootCondition::EntityOnFire
            } else {
                DynamicLootCondition::None
            }
        }
        "weather_check" => {
            let raining = val.get("raining").and_then(Value::as_bool);
            let thundering = val.get("thundering").and_then(Value::as_bool);
            DynamicLootCondition::WeatherCheck {
                raining,
                thundering,
            }
        }
        _ => DynamicLootCondition::None,
    }
}

fn combine_conditions(conditions: Vec<DynamicLootCondition>) -> DynamicLootCondition {
    if conditions.is_empty() {
        DynamicLootCondition::None
    } else if conditions.len() == 1 {
        conditions.into_iter().next().unwrap_or_default()
    } else {
        DynamicLootCondition::AllOf(conditions)
    }
}

/// Loads all loot table JSON files from a directory recursively into `registry`.
pub fn load_loot_tables_from_dir<S: std::hash::BuildHasher>(
    namespace: &str,
    dir: &Path,
    registry: &mut HashMap<String, Arc<DynamicLootTable>, S>,
) -> usize {
    if !dir.is_dir() {
        return 0;
    }
    let before = registry.len();
    load_loot_tables_recursive(namespace, dir, dir, registry);
    registry.len() - before
}

fn load_loot_tables_recursive<S: std::hash::BuildHasher>(
    namespace: &str,
    base_dir: &Path,
    current_dir: &Path,
    registry: &mut HashMap<String, Arc<DynamicLootTable>, S>,
) {
    let Ok(entries) = fs::read_dir(current_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            load_loot_tables_recursive(namespace, base_dir, &path, registry);
        } else if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
            && let Ok(rel_path) = path.strip_prefix(base_dir)
        {
            let mut stem_path = rel_path.to_string_lossy().to_string();
            if let Some(stem) = stem_path.strip_suffix(".json") {
                stem_path = stem.to_string();
            }
            let stem_path = stem_path.replace('\\', "/");
            let loot_table_id = format!("{namespace}:{stem_path}");

            if let Ok(content) = fs::read_to_string(&path)
                && let Some(table) = parse_loot_table(&content)
            {
                registry.insert(loot_table_id, Arc::new(table));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_loot_table() {
        let json = r#"{
            "type": "minecraft:chest",
            "pools": [
                {
                    "rolls": 2,
                    "entries": [
                        {
                            "type": "minecraft:item",
                            "name": "minecraft:diamond",
                            "weight": 5,
                            "functions": [
                                {
                                    "function": "minecraft:set_count",
                                    "count": {"min": 1, "max": 3}
                                }
                            ]
                        },
                        {
                            "type": "minecraft:empty",
                            "weight": 10
                        }
                    ]
                }
            ]
        }"#;

        let table = parse_loot_table(json).expect("valid loot table");
        assert_eq!(table.pools.len(), 1);
        let pool = &table.pools[0];
        assert_eq!(pool.min_rolls, 2);
        assert_eq!(pool.max_rolls, 2);
        assert_eq!(pool.empty_weight, 10);
        assert_eq!(pool.entries.len(), 1);
        assert_eq!(pool.entries[0].item, "minecraft:diamond");
        assert_eq!(pool.entries[0].weight, 5);
        assert_eq!(pool.entries[0].min_count, 1);
        assert_eq!(pool.entries[0].max_count, 3);
    }

    #[test]
    fn parse_block_loot_with_silk_touch_and_fortune() {
        let json = r#"{
            "type": "minecraft:block",
            "pools": [
                {
                    "rolls": 1,
                    "entries": [
                        {
                            "type": "minecraft:alternatives",
                            "children": [
                                {
                                    "type": "minecraft:item",
                                    "name": "minecraft:coal_ore",
                                    "conditions": [
                                        {
                                            "condition": "minecraft:match_tool",
                                            "predicate": {
                                                "predicates": {
                                                    "minecraft:enchantments": [
                                                        {"enchantments": "minecraft:silk_touch"}
                                                    ]
                                                }
                                            }
                                        }
                                    ]
                                },
                                {
                                    "type": "minecraft:item",
                                    "name": "minecraft:coal",
                                    "functions": [
                                        {
                                            "function": "minecraft:apply_bonus",
                                            "formula": "minecraft:ore_drops"
                                        }
                                    ],
                                    "conditions": [
                                        {
                                            "condition": "minecraft:survives_explosion"
                                        }
                                    ]
                                }
                            ]
                        }
                    ]
                }
            ]
        }"#;

        let table = parse_loot_table(json).expect("valid block loot table");
        assert_eq!(table.pools.len(), 1);
        let pool = &table.pools[0];
        assert_eq!(pool.entries.len(), 2);
        assert_eq!(pool.entries[0].item, "minecraft:coal_ore");
        assert_eq!(pool.entries[0].condition, DynamicLootCondition::SilkTouch);
        assert_eq!(pool.entries[1].item, "minecraft:coal");
        assert_eq!(
            pool.entries[1].bonus_formula,
            Some(LootBonusFormula::OreDrops)
        );
        assert_eq!(
            pool.entries[1].condition,
            DynamicLootCondition::SurvivesExplosion
        );
    }

    #[test]
    fn directory_loading() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        let sub = root.join("blocks");
        fs::create_dir_all(&sub).expect("create dir");

        let table_json = r#"{
            "pools": [
                {
                    "rolls": 1,
                    "entries": [
                        {
                            "type": "minecraft:item",
                            "name": "minecraft:dirt"
                        }
                    ]
                }
            ]
        }"#;
        fs::write(sub.join("dirt.json"), table_json).expect("write json");

        let mut registry = HashMap::new();
        let count = load_loot_tables_from_dir("custom", root, &mut registry);
        assert_eq!(count, 1);
        assert!(registry.contains_key("custom:blocks/dirt"));
    }
}
