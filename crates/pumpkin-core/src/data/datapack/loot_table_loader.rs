use pumpkin_util::loot_table::DynamicLootTable;
use pumpkin_util::loot_table::parser::parse_loot_table_with_predicates;
use std::{collections::HashMap, fs, path::Path, sync::Arc};

#[must_use]
pub fn parse_loot_table(json: &str) -> Option<DynamicLootTable> {
    parse_loot_table_with_predicates(json, &|name| {
        pumpkin_data::loot_table::get_loot_predicate(name).cloned()
    })
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
                && let Some(table) = parse_loot_table_with_predicates(&content, &|name| {
                    let id = pumpkin_util::identifier::Identifier::parse(name).ok()?;
                    let (namespace, path) = (id.namespace(), id.path());
                    if !Path::new(namespace)
                        .components()
                        .chain(Path::new(path).components())
                        .all(|part| matches!(part, std::path::Component::Normal(_)))
                    {
                        return None;
                    }
                    let data_dir = base_dir.parent()?.parent()?;
                    let file = data_dir
                        .join(namespace)
                        .join("predicate")
                        .join(format!("{path}.json"));
                    fs::read_to_string(file)
                        .ok()
                        .and_then(|json| serde_json::from_str(&json).ok())
                        .or_else(|| {
                            pumpkin_data::loot_table::get_loot_predicate(&format!(
                                "{namespace}:{path}"
                            ))
                            .cloned()
                        })
                })
            {
                registry.insert(loot_table_id, Arc::new(table));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_util::loot_table::{
        DynamicLootCondition, DynamicLootEntryKind, LootBonusFormula, LootFunctionKind,
        LootNumberProvider,
    };

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
        assert_eq!(pool.rolls, LootNumberProvider::Constant(2.0));
        assert_eq!(pool.entries.len(), 2);
        assert_eq!(pool.entries[1].weight, 10);
        assert!(matches!(pool.entries[1].kind, DynamicLootEntryKind::Empty));
        assert!(
            matches!(&pool.entries[0].kind, DynamicLootEntryKind::Item(name) if name == "minecraft:diamond")
        );
        assert_eq!(pool.entries[0].weight, 5);
        assert!(matches!(
            pool.entries[0].functions[0].kind,
            LootFunctionKind::SetCount {
                count: LootNumberProvider::Uniform { min: 1.0, max: 3.0 },
                add: false
            }
        ));
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
        let DynamicLootEntryKind::Alternatives(children) = &pool.entries[0].kind else {
            panic!("expected alternatives")
        };
        assert_eq!(children.len(), 2);
        assert!(
            matches!(&children[0].kind, DynamicLootEntryKind::Item(name) if name == "minecraft:coal_ore")
        );
        assert!(matches!(
            children[0].condition,
            DynamicLootCondition::MatchTool(_)
        ));
        assert!(
            matches!(&children[1].kind, DynamicLootEntryKind::Item(name) if name == "minecraft:coal")
        );
        assert!(matches!(
            children[1].functions[0].kind,
            LootFunctionKind::ApplyBonus(LootBonusFormula::OreDrops)
        ));
        assert_eq!(
            children[1].condition,
            DynamicLootCondition::SurvivesExplosion
        );
    }

    #[test]
    fn directory_loading() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path().join("data/custom/loot_table");
        let predicates = temp.path().join("data/custom/predicate");
        fs::create_dir_all(&predicates).expect("create predicates");
        fs::write(
            predicates.join("disabled.json"),
            r#"{"type":"minecraft:random_chance","chance":0}"#,
        )
        .expect("write predicate");
        let sub = root.join("blocks");
        fs::create_dir_all(&sub).expect("create dir");

        let table_json = r#"{
            "pools": [
                {
                    "rolls": 1,
                    "condition": "custom:disabled",
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
        let count = load_loot_tables_from_dir("custom", &root, &mut registry);
        assert_eq!(count, 1);
        assert!(matches!(
            registry["custom:blocks/dirt"].pools[0].condition,
            DynamicLootCondition::RandomChance { chance: 0.0 }
        ));
    }
}
