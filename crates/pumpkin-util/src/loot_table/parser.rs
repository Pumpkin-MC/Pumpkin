use serde_json::Value;

use super::{
    DynamicLootCondition as Condition, DynamicLootEntry as Entry,
    DynamicLootEntryKind as EntryKind, DynamicLootFunction as Function, DynamicLootPool as Pool,
    DynamicLootTable as Table, LootBonusFormula, LootEntityTarget, LootFunctionKind,
    LootNumberProvider,
};

const MAX_PREDICATE_DEPTH: usize = 128;

/// Parses loot tables while preserving composite entries and nested table references.
#[must_use]
pub fn parse_loot_table(json: &str) -> Option<Table> {
    parse_loot_table_with_predicates(json, &|_| None)
}

/// Parses a condition with a resolver for named predicate definitions.
#[must_use]
pub fn parse_condition_with_predicates(
    value: &Value,
    predicates: &impl Fn(&str) -> Option<Value>,
) -> Option<Condition> {
    Parser { predicates }.condition(value, 0)
}

/// Parses a condition without predicate resolution.
#[must_use]
pub fn parse_condition(value: &Value) -> Option<Condition> {
    parse_condition_with_predicates(value, &|_| None)
}

/// Parses a loot table with a resolver for named predicate definitions.
pub fn parse_loot_table_with_predicates(
    json: &str,
    predicates: &impl Fn(&str) -> Option<Value>,
) -> Option<Table> {
    Parser { predicates }.table(&serde_json::from_str::<Value>(json).ok()?)
}

struct Parser<'a, R> {
    predicates: &'a R,
}

fn kind(value: &Value) -> &str {
    let name = value
        .get("type")
        .or_else(|| value.get("condition"))
        .or_else(|| value.get("function"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    name.strip_prefix("minecraft:").unwrap_or(name)
}

fn many(value: &Value) -> &[Value] {
    value
        .as_array()
        .map_or_else(|| std::slice::from_ref(value), Vec::as_slice)
}

fn all(conditions: Vec<Condition>) -> Condition {
    match conditions.len() {
        0 => Condition::None,
        1 => conditions.into_iter().next().unwrap_or_default(),
        _ => Condition::AllOf(conditions),
    }
}

fn number(value: &Value) -> Option<i32> {
    value.as_f64().map(|number| number as i32)
}

fn scalar(value: &Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.get("value").and_then(Value::as_f64))
}

fn provider(value: &Value) -> Option<LootNumberProvider> {
    if let Some(value) = scalar(value) {
        return Some(LootNumberProvider::Constant(value));
    }
    match kind(value) {
        "uniform" | "" => Some(LootNumberProvider::Uniform {
            min: scalar(value.get("min")?)?,
            max: scalar(value.get("max")?)?,
        }),
        "binomial" => Some(LootNumberProvider::Binomial {
            n: number(value.get("n")?)?,
            probability: scalar(value.get("p")?)? as f32,
        }),
        _ => None,
    }
}

impl<R: Fn(&str) -> Option<Value>> Parser<'_, R> {
    fn table(&self, value: &Value) -> Option<Table> {
        value.as_object()?;
        let pools = match value.get("pools") {
            Some(pools) => pools
                .as_array()?
                .iter()
                .map(|pool| self.pool(pool))
                .collect::<Option<_>>()?,
            None => Vec::new(),
        };
        Some(Table {
            pools,
            functions: self.functions(value),
        })
    }

    fn pool(&self, value: &Value) -> Option<Pool> {
        let entries = value
            .get("entries")
            .and_then(Value::as_array)?
            .iter()
            .map(|entry| self.entry(entry))
            .collect::<Option<_>>()?;
        let rolls = value
            .get("rolls")
            .map_or(Some(LootNumberProvider::Constant(1.0)), provider)?;
        let bonus_rolls = value
            .get("bonus_rolls")
            .map_or(Some(LootNumberProvider::Constant(0.0)), provider)?;
        Some(Pool {
            entries,
            rolls,
            bonus_rolls,
            condition: self.conditions(value),
            functions: self.functions(value),
        })
    }

    fn entry(&self, value: &Value) -> Option<Entry> {
        let children = || -> Option<Vec<Entry>> {
            match value.get("children") {
                Some(children) => children
                    .as_array()?
                    .iter()
                    .map(|child| self.entry(child))
                    .collect(),
                None => Some(Vec::new()),
            }
        };
        let kind = match kind(value) {
            "item" => EntryKind::Item(value.get("name")?.as_str()?.to_owned()),
            "dynamic" => EntryKind::Dynamic(value.get("name")?.as_str()?.to_owned()),
            "empty" => EntryKind::Empty,
            "alternatives" => EntryKind::Alternatives(children()?),
            "group" => EntryKind::Group(children()?),
            "sequence" => EntryKind::Sequence(children()?),
            "tag" => EntryKind::Tag {
                name: value
                    .get("items")
                    .or_else(|| value.get("name"))?
                    .as_str()?
                    .to_owned(),
                expand: value
                    .get("expand")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            },
            "loot_table" => {
                let table = value.get("value").or_else(|| value.get("name"))?;
                if let Some(name) = table.as_str() {
                    EntryKind::Table(name.to_owned())
                } else {
                    EntryKind::InlineTable(Box::new(self.table(table)?))
                }
            }
            _ => return None,
        };
        Some(Entry {
            kind,
            weight: value
                .get("weight")
                .and_then(Value::as_i64)
                .and_then(|weight| i32::try_from(weight).ok())
                .unwrap_or(1),
            quality: value
                .get("quality")
                .and_then(Value::as_i64)
                .and_then(|value| i32::try_from(value).ok())
                .unwrap_or(0),
            condition: self.conditions(value),
            functions: self.functions(value),
        })
    }

    fn conditions(&self, value: &Value) -> Condition {
        value
            .get("condition")
            .or_else(|| value.get("conditions"))
            .map_or(Condition::None, |value| {
                self.condition(value, 0).unwrap_or(Condition::Never)
            })
    }

    #[allow(clippy::too_many_lines)]
    fn condition(&self, value: &Value, depth: usize) -> Option<Condition> {
        if depth >= MAX_PREDICATE_DEPTH {
            return None;
        }
        if let Some(key) = value.as_str() {
            return (self.predicates)(key).and_then(|value| self.condition(&value, depth + 1));
        }
        if let Some(conditions) = value.as_array() {
            return Some(all(conditions
                .iter()
                .map(|condition| self.condition(condition, depth + 1))
                .collect::<Option<_>>()?));
        }
        Some(match kind(value) {
            "survives_explosion" => Condition::SurvivesExplosion,
            "killed_by_player" => Condition::KilledByPlayer,
            "random_chance" => Condition::RandomChance {
                chance: value.get("chance").and_then(Value::as_f64).unwrap_or(0.0) as f32,
            },
            "random_chance_with_enchanted_bonus" => {
                let unenchanted_chance = value
                    .get("unenchanted_chance")
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0) as f32;
                let chance = value.get("enchanted_chance");
                let enchanted_chance_base = chance
                    .and_then(|chance| {
                        chance
                            .as_f64()
                            .or_else(|| chance.get("base").and_then(Value::as_f64))
                    })
                    .map_or(unenchanted_chance, |number| number as f32);
                let enchanted_chance_per_level_above_first = chance
                    .and_then(|chance| chance.get("per_level_above_first"))
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0)
                    as f32;
                Condition::RandomChanceWithEnchantedBonus {
                    enchantment: value
                        .get("enchantment")
                        .and_then(Value::as_str)
                        .unwrap_or("minecraft:looting")
                        .to_owned(),
                    unenchanted_chance,
                    enchanted_chance_base,
                    enchanted_chance_per_level_above_first,
                }
            }
            "random_chance_with_looting" => {
                let chance = value.get("chance").and_then(Value::as_f64).unwrap_or(0.0) as f32;
                let looting_multiplier = value
                    .get("looting_multiplier")
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0) as f32;
                Condition::RandomChanceWithLooting {
                    chance,
                    looting_multiplier,
                }
            }
            "table_bonus" => Condition::TableBonus {
                enchantment: value
                    .get("enchantment")
                    .and_then(Value::as_str)
                    .unwrap_or("minecraft:fortune")
                    .to_owned(),
                chances: value
                    .get("chances")
                    .and_then(Value::as_array)
                    .map(|values| {
                        values
                            .iter()
                            .filter_map(Value::as_f64)
                            .map(|value| value as f32)
                            .collect()
                    })
                    .unwrap_or_default(),
            },
            "all_of" => all(value
                .get("terms")?
                .as_array()?
                .iter()
                .map(|term| self.condition(term, depth + 1))
                .collect::<Option<_>>()?),
            "any_of" => Condition::AnyOf(
                value
                    .get("terms")?
                    .as_array()?
                    .iter()
                    .map(|term| self.condition(term, depth + 1))
                    .collect::<Option<_>>()?,
            ),
            "inverted" => {
                Condition::Inverted(Box::new(self.condition(value.get("term")?, depth + 1)?))
            }
            "entity_properties" => Self::entity_properties(value)?,
            "damage_source_properties" => Self::damage_source_properties(value),
            "location_check" => Self::location_check(value)?,
            "match_block" | "block_state_property" => {
                Condition::MatchBlock(value.get("predicate").unwrap_or(value).clone())
            }
            "match_tool" => Condition::MatchTool(
                value
                    .get("predicate")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({})),
            ),
            "weather_check" => Condition::WeatherCheck {
                raining: value.get("raining").and_then(Value::as_bool),
                thundering: value.get("thundering").and_then(Value::as_bool),
            },
            "time_check" => {
                let period = value.get("period").and_then(Value::as_u64);
                let value = value.get("value").cloned().unwrap_or(Value::Null);
                Condition::TimeCheck { value, period }
            }
            "value_check" => {
                let val = value.get("value").and_then(provider)?;
                let range = value.get("range").cloned().unwrap_or(Value::Null);
                Condition::ValueCheck { value: val, range }
            }
            "reference" => {
                let name = value.get("name").and_then(Value::as_str)?;
                (self.predicates)(name)
                    .and_then(|value| self.condition(&value, depth + 1))
                    .unwrap_or_else(|| Condition::Reference(name.to_owned()))
            }
            _ => return None,
        })
    }

    fn location_check(value: &Value) -> Option<Condition> {
        let offset = |key| {
            value.get(key).map_or(Some(0), |value| {
                value.as_i64().and_then(|value| i32::try_from(value).ok())
            })
        };
        Some(Condition::LocationCheck {
            offset: [offset("offsetX")?, offset("offsetY")?, offset("offsetZ")?],
            predicate: value
                .get("predicate")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({})),
        })
    }

    fn entity_properties(value: &Value) -> Option<Condition> {
        let target = match value.get("entity").and_then(Value::as_str) {
            Some("this") => LootEntityTarget::This,
            Some("attacker" | "killer") => LootEntityTarget::Attacker,
            Some("direct_attacker" | "direct_killer") => LootEntityTarget::DirectAttacker,
            _ => return None,
        };
        let mut predicate = value.get("predicate")?.as_object()?.clone();
        let mut conditions = Vec::new();
        if target == LootEntityTarget::This {
            for key in ["minecraft:flags", "flags"] {
                if let Some(flags) = predicate.get_mut(key).and_then(Value::as_object_mut)
                    && flags.get("is_on_fire").and_then(Value::as_bool) == Some(true)
                {
                    conditions.push(Condition::EntityOnFire);
                    flags.remove("is_on_fire");
                    if flags.is_empty() {
                        predicate.remove(key);
                    }
                }
            }
        }
        if target == LootEntityTarget::DirectAttacker {
            for key in ["minecraft:equipment", "equipment"] {
                if let Some(tag) = predicate
                    .get(key)
                    .and_then(|equipment| {
                        equipment
                            .pointer("/mainhand/predicates/minecraft:enchantments/0/enchantments")
                    })
                    .and_then(Value::as_str)
                    .and_then(|tag| tag.strip_prefix('#'))
                {
                    conditions.push(Condition::DirectAttackerWeaponEnchantment(tag.to_owned()));
                    predicate.remove(key);
                }
            }
        }
        if !predicate.is_empty() || conditions.is_empty() {
            conditions.push(Condition::EntityProperties {
                target,
                predicate: Value::Object(predicate),
            });
        }
        Some(all(conditions))
    }

    fn damage_source_properties(value: &Value) -> Condition {
        let Some(predicate) = value.get("predicate") else {
            return Condition::Never;
        };
        let mut conditions = Vec::new();
        for (field, target) in [
            ("source_entity", LootEntityTarget::Attacker),
            ("direct_entity", LootEntityTarget::DirectAttacker),
        ] {
            if let Some(predicate) = predicate.get(field) {
                conditions.push(Condition::EntityProperties {
                    target,
                    predicate: predicate.clone(),
                });
            }
        }
        if let Some(tags) = predicate.get("tags").and_then(Value::as_array) {
            for tag in tags {
                if let Some(name) = tag.get("id").and_then(Value::as_str) {
                    conditions.push(Condition::DamageSourceTag {
                        tag: name.trim_start_matches('#').to_owned(),
                        expected: tag.get("expected").and_then(Value::as_bool).unwrap_or(true),
                    });
                }
            }
        }
        for (flag, tag_name) in [
            ("is_projectile", "minecraft:is_projectile"),
            ("is_explosion", "minecraft:is_explosion"),
            ("is_fire", "minecraft:is_fire"),
            ("is_lightning", "minecraft:is_lightning"),
            ("is_fall", "minecraft:is_fall"),
        ] {
            if let Some(expected) = predicate.get(flag).and_then(Value::as_bool) {
                conditions.push(Condition::DamageSourceTag {
                    tag: tag_name.to_owned(),
                    expected,
                });
            }
        }
        all(conditions)
    }

    fn functions(&self, value: &Value) -> Vec<Function> {
        let Some(functions) = value.get("modifier").or_else(|| value.get("functions")) else {
            return Vec::new();
        };
        many(functions)
            .iter()
            .filter_map(|function| {
                let kind = match kind(function) {
                    "set_count" => {
                        let count = provider(function.get("count")?)?;
                        LootFunctionKind::SetCount {
                            count,
                            add: function
                                .get("add")
                                .and_then(Value::as_bool)
                                .unwrap_or(false),
                        }
                    }
                    "furnace_smelt" => LootFunctionKind::FurnaceSmelt,
                    "apply_bonus" => {
                        let formula = function.get("formula").and_then(Value::as_str)?;
                        let parameters = function.get("parameters");
                        let formula = match formula.trim_start_matches("minecraft:") {
                            "ore_drops" => LootBonusFormula::OreDrops,
                            "uniform_bonus_count" => LootBonusFormula::UniformBonusCount(
                                parameters
                                    .and_then(|parameters| parameters.get("bonusMultiplier"))
                                    .and_then(number)
                                    .unwrap_or(1),
                            ),
                            "binomial_with_bonus_count" => {
                                LootBonusFormula::BinomialWithBonusCount {
                                    extra: parameters
                                        .and_then(|parameters| parameters.get("extra"))
                                        .and_then(number)
                                        .unwrap_or(0),
                                    probability: parameters
                                        .and_then(|parameters| parameters.get("probability"))
                                        .and_then(Value::as_f64)
                                        .unwrap_or(0.0)
                                        as f32,
                                }
                            }
                            _ => return None,
                        };
                        LootFunctionKind::ApplyBonus(formula)
                    }
                    "enchanted_count_increase" | "looting_enchant" => {
                        LootFunctionKind::EnchantedCountIncrease {
                            count: provider(function.get("count")?)?,
                            limit: function.get("limit").and_then(number).unwrap_or(0),
                        }
                    }
                    "limit_count" => {
                        let limit = function.get("limit")?;
                        let (min, max) = if let Some(value) = scalar(limit) {
                            (
                                Some(LootNumberProvider::Constant(value)),
                                Some(LootNumberProvider::Constant(value)),
                            )
                        } else {
                            (
                                match limit.get("min") {
                                    Some(value) => Some(provider(value)?),
                                    None => None,
                                },
                                match limit.get("max") {
                                    Some(value) => Some(provider(value)?),
                                    None => None,
                                },
                            )
                        };
                        LootFunctionKind::LimitCount { min, max }
                    }
                    "explosion_decay" => LootFunctionKind::ExplosionDecay,
                    _ => return None,
                };
                let enchantment = function
                    .get("enchantment")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
                    .or_else(|| match kind {
                        LootFunctionKind::ApplyBonus(_) => Some("minecraft:fortune".to_owned()),
                        LootFunctionKind::EnchantedCountIncrease { .. } => {
                            Some("minecraft:looting".to_owned())
                        }
                        _ => None,
                    });
                Some(Function {
                    kind,
                    enchantment,
                    condition: self.conditions(function),
                })
            })
            .collect()
    }
}
