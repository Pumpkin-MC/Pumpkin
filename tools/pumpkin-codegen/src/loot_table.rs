use std::{collections::BTreeMap, fs, path::Path};

use heck::ToShoutySnakeCase;
use proc_macro2::{Span, TokenStream};
use pumpkin_util::loot_table::{
    LootBonusFormula, LootCondition, LootEntityPredicate, LootEntityProperty,
    LootEntityPropertyValue, LootEntityTarget,
};
use quote::{format_ident, quote};
use serde::Deserialize;
use syn::LitStr;

/// `rolls` can be a bare float or an object with `type/min/max`.
#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
enum RollsStruct {
    Constant(f32),
    Provider {
        #[allow(dead_code)]
        #[serde(rename = "type")]
        provider_type: String,
        #[allow(dead_code)]
        #[serde(default)]
        min: f32,
        #[serde(default)]
        max: f32,
    },
}

impl RollsStruct {
    fn min(&self) -> i32 {
        match self {
            Self::Constant(v) => v.round() as i32,
            Self::Provider { min, .. } => min.round() as i32,
        }
    }
    fn max(&self) -> i32 {
        match self {
            Self::Constant(v) => v.round() as i32,
            Self::Provider { max, .. } => max.round() as i32,
        }
    }
}

/// A `set_count` count provider (uniform or constant).
#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
enum CountStruct {
    Constant(f32),
    Provider {
        #[serde(rename = "type")]
        #[allow(dead_code)]
        provider_type: String,
        #[serde(default)]
        min: f32,
        #[serde(default)]
        max: f32,
    },
}

impl CountStruct {
    fn min(&self) -> i32 {
        match self {
            Self::Constant(v) => v.round() as i32,
            Self::Provider { min, .. } => min.round() as i32,
        }
    }
    fn max(&self) -> i32 {
        match self {
            Self::Constant(v) => v.round() as i32,
            Self::Provider { max, .. } => max.round() as i32,
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
struct PredicateStruct {
    #[serde(default)]
    items: Option<serde_json::Value>,
    #[serde(default)]
    predicates: Option<serde_json::Value>,
    #[serde(flatten)]
    entity_properties: BTreeMap<String, serde_json::Value>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
enum EnchantedChanceStruct {
    Constant(f32),
    Linear {
        #[serde(rename = "type")]
        chance_type: String,
        base: f32,
        #[serde(default)]
        per_level_above_first: f32,
    },
}

/// A condition is either an inline definition or a reference to a `predicate` file.
#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
enum ConditionValue {
    Reference(String),
    Inline(Box<ConditionStruct>),
}

#[derive(Deserialize, Clone, Debug)]
struct ConditionStruct {
    #[serde(rename = "type", default)]
    condition: String,
    #[allow(dead_code)]
    #[serde(default)]
    enchantment: Option<String>,
    #[serde(default)]
    chance: Option<f32>,
    #[serde(default)]
    unenchanted_chance: Option<f32>,
    #[serde(default)]
    enchanted_chance: Option<EnchantedChanceStruct>,
    #[serde(default)]
    chances: Option<Vec<f32>>,
    #[serde(default)]
    predicate: Option<PredicateStruct>,
    #[serde(default)]
    entity: Option<String>,
    #[serde(default)]
    term: Option<ConditionValue>,
    #[serde(default)]
    terms: Option<Vec<ConditionValue>>,
}

fn resolve_condition(value: &ConditionValue) -> LootCondition {
    match value {
        ConditionValue::Inline(cond) => parse_condition(cond),
        ConditionValue::Reference(name) => {
            let relative = name.strip_prefix("minecraft:").unwrap_or(name);
            let path = Path::new("../../assets/datapacks/26_3/data/minecraft/predicate")
                .join(format!("{relative}.json"));
            fs::read_to_string(&path)
                .ok()
                .and_then(|content| serde_json::from_str::<ConditionStruct>(&content).ok())
                .map_or(LootCondition::None, |cond| parse_condition(&cond))
        }
    }
}

/// Translate one vanilla loot condition into its generated [`LootCondition`].
///
/// Conditions the runtime cannot evaluate are mapped to [`LootCondition::None`], which keeps
/// the owning entry unconditional rather than dropping it.
fn parse_condition(cond: &ConditionStruct) -> LootCondition {
    match cond.condition.as_str() {
        "minecraft:survives_explosion" => LootCondition::SurvivesExplosion,
        "minecraft:killed_by_player" => LootCondition::KilledByPlayer,
        "minecraft:random_chance" => {
            let chance = cond
                .chance
                .or_else(|| cond.chances.as_ref().and_then(|c| c.first().copied()))
                .unwrap_or(0.0);
            LootCondition::RandomChance { chance }
        }
        "minecraft:random_chance_with_enchanted_bonus" => {
            let unenchanted_chance = cond.unenchanted_chance.unwrap_or(0.0);
            let (enchanted_chance_base, enchanted_chance_per_level_above_first) =
                match &cond.enchanted_chance {
                    Some(EnchantedChanceStruct::Linear {
                        base,
                        per_level_above_first,
                        ..
                    }) => (*base, *per_level_above_first),
                    Some(EnchantedChanceStruct::Constant(c)) => (*c, 0.0),
                    None => (unenchanted_chance, 0.0),
                };
            LootCondition::RandomChanceWithEnchantedBonus {
                unenchanted_chance,
                enchanted_chance_base,
                enchanted_chance_per_level_above_first,
            }
        }
        "minecraft:entity_properties" => {
            let target = match cond.entity.as_deref() {
                Some("this") => LootEntityTarget::This,
                Some("attacker") => LootEntityTarget::Attacker,
                Some("direct_attacker") => LootEntityTarget::DirectAttacker,
                _ => return LootCondition::None,
            };

            let Some(predicate) = &cond.predicate else {
                return LootCondition::None;
            };

            let mut properties = Vec::new();
            for (key, value) in &predicate.entity_properties {
                flatten_entity_properties(key, value, &mut properties);
            }
            if properties.is_empty() {
                return LootCondition::None;
            }

            LootCondition::EntityProperties {
                target,
                predicate: LootEntityPredicate {
                    properties: Box::leak(properties.into_boxed_slice()),
                },
            }
        }
        "minecraft:table_bonus" => {
            let chances = cond.chances.clone().unwrap_or_default();
            if chances.is_empty() {
                LootCondition::None
            } else {
                LootCondition::TableBonus {
                    chances: Box::leak(chances.into_boxed_slice()),
                }
            }
        }
        "minecraft:all_of" => {
            if let Some(terms) = &cond.terms {
                combine_conditions(terms)
            } else {
                LootCondition::None
            }
        }
        "minecraft:match_tool" => {
            if let Some(pred) = &cond.predicate {
                if let Some(items_val) = &pred.items {
                    let is_shears = match items_val {
                        serde_json::Value::String(s) => s.contains("shears"),
                        serde_json::Value::Array(arr) => arr
                            .iter()
                            .any(|v| v.as_str().is_some_and(|s| s.contains("shears"))),
                        _ => false,
                    };
                    if is_shears {
                        return LootCondition::Shears;
                    }
                }
                if let Some(pred_val) = &pred.predicates {
                    let s = pred_val.to_string();
                    if s.contains("silk_touch") {
                        return LootCondition::SilkTouch;
                    }
                }
            }
            LootCondition::None
        }
        "minecraft:any_of" => {
            if let Some(terms) = &cond.terms {
                let has_silk = terms
                    .iter()
                    .any(|t| resolve_condition(t) == LootCondition::SilkTouch);
                let has_shears = terms
                    .iter()
                    .any(|t| resolve_condition(t) == LootCondition::Shears);
                if has_silk && has_shears {
                    return LootCondition::SilkTouchOrShears;
                } else if has_silk {
                    return LootCondition::SilkTouch;
                } else if has_shears {
                    return LootCondition::Shears;
                }
            }
            LootCondition::None
        }
        "minecraft:inverted" => {
            if let Some(term) = &cond.term {
                match resolve_condition(term) {
                    LootCondition::SilkTouch => LootCondition::NoSilkTouch,
                    LootCondition::Shears => LootCondition::NoSilkTouchOrShears,
                    LootCondition::SilkTouchOrShears => LootCondition::NoSilkTouchOrShears,
                    _ => LootCondition::None,
                }
            } else {
                LootCondition::None
            }
        }
        _ => LootCondition::None,
    }
}

/// Flatten a nested entity predicate into `path/to/key` and primitive value pairs.
///
/// `path` is the prefix accumulated so far; nested objects recurse with their key appended.
/// Values the runtime has no representation for (arrays, floats, `null`) are skipped, so a
/// predicate that only contains those ends up empty and its condition becomes
/// [`LootCondition::None`].
fn flatten_entity_properties(
    path: &str,
    value: &serde_json::Value,
    properties: &mut Vec<LootEntityProperty>,
) {
    match value {
        serde_json::Value::Object(values) => {
            for (key, value) in values {
                flatten_entity_properties(&format!("{path}/{key}"), value, properties);
            }
        }
        serde_json::Value::Bool(value) => properties.push(LootEntityProperty {
            key: Box::leak(path.to_owned().into_boxed_str()),
            value: LootEntityPropertyValue::Bool(*value),
        }),
        serde_json::Value::Number(value) => {
            if let Some(value) = value.as_i64() {
                properties.push(LootEntityProperty {
                    key: Box::leak(path.to_owned().into_boxed_str()),
                    value: LootEntityPropertyValue::Integer(value),
                });
            }
        }
        serde_json::Value::String(value) => properties.push(LootEntityProperty {
            key: Box::leak(path.to_owned().into_boxed_str()),
            value: LootEntityPropertyValue::String(Box::leak(value.clone().into_boxed_str())),
        }),
        _ => {}
    }
}

/// Resolve an optional condition, treating a missing one as [`LootCondition::None`].
fn condition_of(value: Option<&ConditionValue>) -> LootCondition {
    value.map_or(LootCondition::None, resolve_condition)
}

/// Combine every condition on a pool or entry into a single [`LootCondition`].
///
/// Unrepresentable conditions drop out, a single condition is returned as-is, and two or more
/// are wrapped in [`LootCondition::AllOf`].
fn combine_conditions(conditions: &[ConditionValue]) -> LootCondition {
    let mut parsed_list: Vec<LootCondition> = Vec::new();
    for c in conditions {
        let parsed = resolve_condition(c);
        if parsed != LootCondition::None {
            parsed_list.push(parsed);
        }
    }
    match parsed_list.len() {
        0 => LootCondition::None,
        1 => parsed_list[0],
        _ => LootCondition::AllOf(Box::leak(parsed_list.into_boxed_slice())),
    }
}

#[derive(Deserialize, Clone, Debug)]
struct BonusParameterStruct {
    #[serde(rename = "bonusMultiplier", default)]
    bonus_multiplier: Option<i32>,
    #[serde(default)]
    extra: Option<i32>,
    #[serde(default)]
    probability: Option<f32>,
}

/// Item modifiers hold either a single entry or a list of them.
pub fn one_or_many<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    // `Many` is tried first because `serde_json::Value` would also match `One`.
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OneOrMany<T> {
        Many(Vec<T>),
        One(T),
    }

    Ok(match OneOrMany::deserialize(deserializer)? {
        OneOrMany::Many(values) => values,
        OneOrMany::One(value) => vec![value],
    })
}

#[derive(Deserialize, Clone, Debug)]
struct EntryFunctionStruct {
    #[serde(rename = "type")]
    function: String,
    #[serde(default)]
    formula: Option<String>,
    #[serde(default)]
    parameters: Option<BonusParameterStruct>,
    count: Option<CountStruct>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
enum LootTableValue {
    Reference(String),
    Inline(ChestLootTableJson),
}

#[derive(Deserialize, Clone, Debug)]
struct PoolEntryStruct {
    #[serde(rename = "type")]
    entry_type: String,
    name: Option<String>,
    /// Tag entries name their tag here.
    #[serde(default)]
    items: Option<String>,
    #[serde(default)]
    value: Option<LootTableValue>,
    #[serde(default = "default_weight")]
    weight: i32,
    #[serde(rename = "modifier", default, deserialize_with = "one_or_many")]
    functions: Vec<EntryFunctionStruct>,
    #[serde(default)]
    condition: Option<ConditionValue>,
    #[serde(default)]
    children: Vec<PoolEntryStruct>,
}

fn default_weight() -> i32 {
    1
}

#[derive(Deserialize, Clone, Debug)]
struct PoolStruct {
    #[serde(default)]
    entries: Vec<PoolEntryStruct>,
    #[serde(default = "default_rolls")]
    rolls: RollsStruct,
    #[serde(default)]
    condition: Option<ConditionValue>,
}

fn default_rolls() -> RollsStruct {
    RollsStruct::Constant(1.0)
}

#[derive(Deserialize, Clone, Debug)]
struct ChestLootTableJson {
    #[serde(default)]
    pools: Vec<PoolStruct>,
}

fn path_to_key(relative: &str) -> String {
    format!("minecraft:{relative}")
}

fn path_to_ident(relative: &str) -> String {
    relative.replace('/', "_").to_shouty_snake_case()
}

struct ParsedEntry {
    item: String,
    weight: i32,
    min_count: i32,
    max_count: i32,
    condition: LootCondition,
    bonus_formula: Option<LootBonusFormula>,
}

/// Require both conditions, collapsing the cases where one of them adds nothing.
fn combine_pair(first: LootCondition, second: LootCondition) -> LootCondition {
    match (first, second) {
        (LootCondition::None, cond) | (cond, LootCondition::None) => cond,
        (first, second) if first == second => first,
        (first, second) => LootCondition::AllOf(Box::leak(vec![first, second].into_boxed_slice())),
    }
}

/// Flatten one pool entry into the concrete item entries it can produce.
fn extract_entries(
    entry: &PoolEntryStruct,
    inherited_condition: LootCondition,
    out: &mut Vec<ParsedEntry>,
    empty_weight: &mut i32,
) {
    extract_entries_with_depth(entry, inherited_condition, out, empty_weight, 0);
}

/// Recursive worker behind [`extract_entries`], bounded to five levels of nesting.
///
/// `inherited_condition` carries the conditions of every enclosing pool and entry; it is
/// combined with the entry's own conditions so a child can never drop loot while one of its
/// parents' conditions is false.
fn extract_entries_with_depth(
    entry: &PoolEntryStruct,
    inherited_condition: LootCondition,
    out: &mut Vec<ParsedEntry>,
    empty_weight: &mut i32,
    depth: usize,
) {
    if depth > 5 {
        return;
    }

    let entry_cond = combine_pair(inherited_condition, condition_of(entry.condition.as_ref()));

    match entry.entry_type.as_str() {
        "minecraft:empty" => {
            *empty_weight += entry.weight;
        }
        "minecraft:item" => {
            if let Some(name) = &entry.name {
                let (min_count, max_count) = entry
                    .functions
                    .iter()
                    .find(|f| f.function == "minecraft:set_count")
                    .and_then(|f| f.count.as_ref())
                    .map(|c| (c.min(), c.max()))
                    .unwrap_or((1, 1));

                let bonus_formula = entry.functions.iter().find_map(|f| {
                    if f.function == "minecraft:apply_bonus" {
                        match f.formula.as_deref() {
                            Some("minecraft:ore_drops") => Some(LootBonusFormula::OreDrops),
                            Some("minecraft:uniform_bonus_count") => {
                                let mult = f
                                    .parameters
                                    .as_ref()
                                    .and_then(|p| p.bonus_multiplier)
                                    .unwrap_or(1);
                                Some(LootBonusFormula::UniformBonusCount(mult))
                            }
                            Some("minecraft:binomial_with_bonus_count") => {
                                let extra =
                                    f.parameters.as_ref().and_then(|p| p.extra).unwrap_or(0);
                                let prob = f
                                    .parameters
                                    .as_ref()
                                    .and_then(|p| p.probability)
                                    .unwrap_or(0.0);
                                Some(LootBonusFormula::BinomialWithBonusCount {
                                    extra,
                                    probability: prob,
                                })
                            }
                            _ => None,
                        }
                    } else if f.function == "minecraft:enchanted_count_increase" {
                        let mult = f.count.as_ref().map_or(1, |c| c.max());
                        Some(LootBonusFormula::UniformBonusCount(mult))
                    } else {
                        None
                    }
                });

                out.push(ParsedEntry {
                    item: name.clone(),
                    weight: entry.weight,
                    min_count,
                    max_count,
                    condition: entry_cond,
                    bonus_formula,
                });
            }
        }
        "minecraft:tag" => {
            let tag_name_opt = entry
                .items
                .as_deref()
                .or(entry.name.as_deref())
                .or_else(|| match &entry.value {
                    Some(LootTableValue::Reference(r)) => Some(r.as_str()),
                    _ => None,
                });
            if let Some(tag_name) = tag_name_opt {
                let tag_name = tag_name.strip_prefix('#').unwrap_or(tag_name);
                let tag_rel = tag_name.strip_prefix("minecraft:").unwrap_or(tag_name);
                let tag_path = Path::new("../../assets/datapacks/26_3/data/minecraft/tags/item")
                    .join(format!("{tag_rel}.json"));
                if let Ok(content) = fs::read_to_string(&tag_path) {
                    #[derive(Deserialize)]
                    struct TagJson {
                        values: Vec<String>,
                    }
                    if let Ok(tag_data) = serde_json::from_str::<TagJson>(&content) {
                        for item_name in tag_data.values {
                            out.push(ParsedEntry {
                                item: item_name,
                                weight: entry.weight,
                                min_count: 1,
                                max_count: 1,
                                condition: entry_cond,
                                bonus_formula: None,
                            });
                        }
                    }
                }
            }
        }
        "minecraft:loot_table" => match &entry.value {
            Some(LootTableValue::Reference(table_name)) => {
                let table_rel = table_name.strip_prefix("minecraft:").unwrap_or(table_name);
                let table_path = Path::new("../../assets/datapacks/26_3/data/minecraft/loot_table")
                    .join(format!("{table_rel}.json"));
                if let Ok(content) = fs::read_to_string(&table_path) {
                    if let Ok(nested_table) = serde_json::from_str::<ChestLootTableJson>(&content) {
                        for pool in &nested_table.pools {
                            let mut pool_cond = entry_cond;
                            let parsed = condition_of(pool.condition.as_ref());
                            if parsed != LootCondition::None {
                                pool_cond = parsed;
                            }
                            for child_entry in &pool.entries {
                                extract_entries_with_depth(
                                    child_entry,
                                    pool_cond,
                                    out,
                                    empty_weight,
                                    depth + 1,
                                );
                            }
                        }
                    }
                }
            }
            Some(LootTableValue::Inline(nested_table)) => {
                for pool in &nested_table.pools {
                    let mut pool_cond = entry_cond;
                    let parsed = condition_of(pool.condition.as_ref());
                    if parsed != LootCondition::None {
                        pool_cond = parsed;
                    }
                    for child_entry in &pool.entries {
                        extract_entries_with_depth(
                            child_entry,
                            pool_cond,
                            out,
                            empty_weight,
                            depth + 1,
                        );
                    }
                }
            }
            None => {
                if let Some(name) = &entry.name {
                    let table_rel = name.strip_prefix("minecraft:").unwrap_or(name);
                    let table_path =
                        Path::new("../../assets/datapacks/26_3/data/minecraft/loot_table")
                            .join(format!("{table_rel}.json"));
                    if let Ok(content) = fs::read_to_string(&table_path) {
                        if let Ok(nested_table) =
                            serde_json::from_str::<ChestLootTableJson>(&content)
                        {
                            for pool in &nested_table.pools {
                                let mut pool_cond = entry_cond;
                                let parsed = condition_of(pool.condition.as_ref());
                                if parsed != LootCondition::None {
                                    pool_cond = parsed;
                                }
                                for child_entry in &pool.entries {
                                    extract_entries_with_depth(
                                        child_entry,
                                        pool_cond,
                                        out,
                                        empty_weight,
                                        depth + 1,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        },
        "minecraft:alternatives" => {
            let mut saw_silk = false;
            let mut saw_shears = false;

            for child in &entry.children {
                let child_cond = condition_of(child.condition.as_ref());

                // A child gated on a tool re-derives that condition from its own conditions in
                // the recursive call, so only the implicit gate for later children is added
                // here. `entry_cond` always rides along: a child of the alternatives entry can
                // never drop loot while one of its parents' conditions is false.
                let tool_gate = if child_cond == LootCondition::SilkTouch {
                    saw_silk = true;
                    LootCondition::None
                } else if child_cond == LootCondition::Shears {
                    saw_shears = true;
                    LootCondition::None
                } else if child_cond == LootCondition::SilkTouchOrShears {
                    saw_silk = true;
                    saw_shears = true;
                    LootCondition::None
                } else if saw_silk && saw_shears {
                    LootCondition::NoSilkTouchOrShears
                } else if saw_silk {
                    LootCondition::NoSilkTouch
                } else if saw_shears {
                    LootCondition::NoSilkTouchOrShears
                } else {
                    LootCondition::None
                };

                extract_entries_with_depth(
                    child,
                    combine_pair(entry_cond, tool_gate),
                    out,
                    empty_weight,
                    depth + 1,
                );
            }
        }
        "minecraft:sequence" | "minecraft:group" => {
            for child in &entry.children {
                extract_entries_with_depth(child, entry_cond, out, empty_weight, depth + 1);
            }
        }
        _ => {}
    }
}

/// Emit the `const`-compatible token stream that rebuilds `cond` in the generated data.
fn condition_to_tokens(cond: LootCondition) -> TokenStream {
    match cond {
        LootCondition::None => quote! { LootCondition::None },
        LootCondition::SilkTouch => quote! { LootCondition::SilkTouch },
        LootCondition::NoSilkTouch => quote! { LootCondition::NoSilkTouch },
        LootCondition::Shears => quote! { LootCondition::Shears },
        LootCondition::SilkTouchOrShears => quote! { LootCondition::SilkTouchOrShears },
        LootCondition::NoSilkTouchOrShears => quote! { LootCondition::NoSilkTouchOrShears },
        LootCondition::SurvivesExplosion => quote! { LootCondition::SurvivesExplosion },
        LootCondition::KilledByPlayer => quote! { LootCondition::KilledByPlayer },
        LootCondition::RandomChance { chance } => {
            quote! { LootCondition::RandomChance { chance: #chance } }
        }
        LootCondition::RandomChanceWithEnchantedBonus {
            unenchanted_chance,
            enchanted_chance_base,
            enchanted_chance_per_level_above_first,
        } => {
            quote! {
                LootCondition::RandomChanceWithEnchantedBonus {
                    unenchanted_chance: #unenchanted_chance,
                    enchanted_chance_base: #enchanted_chance_base,
                    enchanted_chance_per_level_above_first: #enchanted_chance_per_level_above_first,
                }
            }
        }
        LootCondition::EntityProperties { target, predicate } => {
            let target_tokens = match target {
                LootEntityTarget::This => quote! { LootEntityTarget::This },
                LootEntityTarget::Attacker => quote! { LootEntityTarget::Attacker },
                LootEntityTarget::DirectAttacker => quote! { LootEntityTarget::DirectAttacker },
            };
            let property_tokens = predicate.properties.iter().map(|property| {
                let key = LitStr::new(property.key, Span::call_site());
                let value = match property.value {
                    LootEntityPropertyValue::Bool(value) => {
                        quote! { LootEntityPropertyValue::Bool(#value) }
                    }
                    LootEntityPropertyValue::Integer(value) => {
                        quote! { LootEntityPropertyValue::Integer(#value) }
                    }
                    LootEntityPropertyValue::String(value) => {
                        let value = LitStr::new(value, Span::call_site());
                        quote! { LootEntityPropertyValue::String(#value) }
                    }
                };
                quote! { LootEntityProperty { key: #key, value: #value } }
            });
            quote! {
                LootCondition::EntityProperties {
                    target: #target_tokens,
                    predicate: LootEntityPredicate {
                        properties: &[#(#property_tokens),*],
                    },
                }
            }
        }
        LootCondition::TableBonus { chances } => {
            let values = chances.iter();
            quote! { LootCondition::TableBonus { chances: &[#(#values),*] } }
        }
        LootCondition::AllOf(list) => {
            let tokens: Vec<TokenStream> = list.iter().copied().map(condition_to_tokens).collect();
            quote! { LootCondition::AllOf(&[#(#tokens),*]) }
        }
    }
}

fn bonus_to_tokens(bonus: Option<LootBonusFormula>) -> TokenStream {
    match bonus {
        None => quote! { None },
        Some(LootBonusFormula::OreDrops) => {
            quote! { Some(LootBonusFormula::OreDrops) }
        }
        Some(LootBonusFormula::UniformBonusCount(mult)) => {
            quote! { Some(LootBonusFormula::UniformBonusCount(#mult)) }
        }
        Some(LootBonusFormula::BinomialWithBonusCount { extra, probability }) => {
            quote! { Some(LootBonusFormula::BinomialWithBonusCount { extra: #extra, probability: #probability }) }
        }
    }
}

/// Emit static entry arrays and pool literals for one table.
/// Returns the list of `LootPool` literals (one per pool).
fn emit_table(
    prefix: &str,
    table: &ChestLootTableJson,
    tokens: &mut TokenStream,
) -> Vec<TokenStream> {
    let mut pool_literals = Vec::new();

    for (pool_idx, pool) in table.pools.iter().enumerate() {
        let min_rolls = pool.rolls.min();
        let max_rolls = pool.rolls.max();

        let pool_cond = condition_of(pool.condition.as_ref());

        let mut parsed_entries = Vec::new();
        let mut empty_weight: i32 = 0;

        for entry in &pool.entries {
            extract_entries(
                entry,
                LootCondition::None,
                &mut parsed_entries,
                &mut empty_weight,
            );
        }

        let entry_literals: Vec<TokenStream> = parsed_entries
            .iter()
            .map(|e| {
                let name_lit = LitStr::new(&e.item, Span::call_site());
                let weight = e.weight;
                let min_count = e.min_count;
                let max_count = e.max_count;
                let cond_tokens = condition_to_tokens(e.condition);
                let bonus_tokens = bonus_to_tokens(e.bonus_formula);

                quote! {
                    LootEntry {
                        item: #name_lit,
                        weight: #weight,
                        min_count: #min_count,
                        max_count: #max_count,
                        condition: #cond_tokens,
                        bonus_formula: #bonus_tokens,
                    }
                }
            })
            .collect();

        // Emit the entries static array.
        let entries_ident = format_ident!("{}_POOL{}_ENTRIES", prefix, pool_idx);
        tokens.extend(quote! {
            static #entries_ident: &[LootEntry] = &[#(#entry_literals),*];
        });

        let pool_cond_tokens = condition_to_tokens(pool_cond);

        pool_literals.push(quote! {
            LootPool {
                entries: #entries_ident,
                min_rolls: #min_rolls,
                max_rolls: #max_rolls,
                empty_weight: #empty_weight,
                condition: #pool_cond_tokens,
            }
        });
    }

    pool_literals
}

/// Recursively collect all `*.json` files under `dir`, returning a vec of
/// `(relative_stem_path, parsed_table)`.
fn collect_json_files(base: &Path, dir: &Path) -> Vec<(String, ChestLootTableJson)> {
    let mut result = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => panic!("failed to read directory {}: {e}", dir.display()),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            result.extend(collect_json_files(base, &path));
        } else if path.extension().is_some_and(|ext| ext == "json") {
            let relative = path
                .strip_prefix(base)
                .unwrap()
                .with_extension("")
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/");

            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => panic!("failed to read {}: {e}", path.display()),
            };

            let table: ChestLootTableJson = match serde_json::from_str(&content) {
                Ok(t) => t,
                Err(e) => panic!("failed to parse {}: {e}", path.display()),
            };

            result.push((relative, table));
        }
    }

    result
}

/// Read every loot JSON from `../../assets/datapacks/26_3/data/minecraft/loot_table/` (recursively)
/// and emit a `pumpkin-data/src/generated/chest_loot.rs` with static constants
/// and a `get_chest_loot_table(key) -> Option<&'static ChestLootTable>` function.
pub fn build() -> TokenStream {
    let base = Path::new("../../assets/datapacks/26_3/data/minecraft/loot_table");

    // Collect all JSON files recursively, sorted for deterministic output.
    let mut files: Vec<(String, ChestLootTableJson)> = collect_json_files(base, base);
    files.sort_by(|a, b| a.0.cmp(&b.0));

    let mut all_tokens = TokenStream::new();

    // Emit one set of statics per file
    let mut table_idents = Vec::new();
    let mut table_keys = Vec::new();
    let mut short_table_keys = Vec::new();

    for (relative_path, table) in &files {
        let prefix = path_to_ident(relative_path);
        let key = path_to_key(relative_path);
        let table_ident = format_ident!("{}", prefix);

        let pool_tokens = emit_table(&prefix, table, &mut all_tokens);

        let pools_ident = format_ident!("{}_POOLS", prefix);
        all_tokens.extend(quote! {
            static #pools_ident: &[LootPool] = &[#(#pool_tokens),*];
            pub static #table_ident: LootTable = LootTable { pools: #pools_ident };
        });

        table_idents.push(table_ident.clone());
        table_keys.push(LitStr::new(&key, Span::call_site()));
        short_table_keys.push(LitStr::new(relative_path, Span::call_site()));
    }

    // Emit get_loot_table and get_chest_loot_table
    all_tokens.extend(quote! {
        #[must_use]
        pub fn get_loot_table(key: &str) -> Option<&'static LootTable> {
            match key {
                #(#table_keys | #short_table_keys => Some(&#table_idents),)*
                _ => None,
            }
        }

        #[must_use]
        pub fn get_chest_loot_table(key: &str) -> Option<&'static LootTable> {
            get_loot_table(key)
        }
    });

    quote! {
        pub use pumpkin_util::loot_table::*;
        #all_tokens
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_util::loot_table::{
        LootEntityPredicate, LootEntityProperty, LootEntityPropertyValue, LootEntityTarget,
    };

    use super::{LootCondition, PoolEntryStruct, extract_entries};

    /// The condition the `entity_properties` child in these fixtures parses into.
    const SHEEP_IS_WHITE: LootCondition = LootCondition::EntityProperties {
        target: LootEntityTarget::This,
        predicate: LootEntityPredicate {
            properties: &[LootEntityProperty {
                key: "minecraft:components/minecraft:sheep/color",
                value: LootEntityPropertyValue::String("white"),
            }],
        },
    };

    /// The implicit "no silk touch" gate a tool-gated sibling creates must not replace the
    /// inherited condition either: a later entity-properties child still has to satisfy its
    /// parent.
    #[test]
    fn alternatives_child_after_a_tool_gate_keeps_inherited_condition() {
        let entry: PoolEntryStruct = serde_json::from_str(
            r#"{
                "type": "minecraft:alternatives",
                "condition": { "type": "minecraft:killed_by_player" },
                "children": [
                    {
                        "type": "minecraft:item",
                        "name": "minecraft:white_wool",
                        "condition": {
                            "type": "minecraft:match_tool",
                            "predicate": {
                                "predicates": {
                                    "minecraft:enchantments": [
                                        { "enchantments": "minecraft:silk_touch" }
                                    ]
                                }
                            }
                        }
                    },
                    {
                        "type": "minecraft:item",
                        "name": "minecraft:white_carpet",
                        "condition": {
                            "type": "minecraft:entity_properties",
                            "entity": "this",
                            "predicate": {
                                "minecraft:components": { "minecraft:sheep/color": "white" }
                            }
                        }
                    }
                ]
            }"#,
        )
        .expect("alternatives entry should deserialize");

        let mut entries = Vec::new();
        let mut empty_weight = 0;
        extract_entries(&entry, LootCondition::None, &mut entries, &mut empty_weight);

        assert_eq!(entries.len(), 2);

        // The silk-touch child keeps both its own gate and the parent condition.
        assert_eq!(entries[0].item, "minecraft:white_wool");
        assert_eq!(
            entries[0].condition,
            LootCondition::AllOf(&[LootCondition::KilledByPlayer, LootCondition::SilkTouch])
        );

        // The child after it gets the implicit "no silk touch" gate on top of both.
        assert_eq!(entries[1].item, "minecraft:white_carpet");
        assert_eq!(
            entries[1].condition,
            LootCondition::AllOf(&[
                LootCondition::AllOf(&[LootCondition::KilledByPlayer, LootCondition::NoSilkTouch,]),
                SHEEP_IS_WHITE,
            ])
        );
    }

    /// Children of `minecraft:alternatives` must keep the condition inherited from the
    /// alternatives entry itself. Dropping it would let an entity-property child drop loot
    /// while its parent condition is false.
    #[test]
    fn alternatives_child_keeps_inherited_condition() {
        let entry: PoolEntryStruct = serde_json::from_str(
            r#"{
                "type": "minecraft:alternatives",
                "condition": { "type": "minecraft:killed_by_player" },
                "children": [
                    {
                        "type": "minecraft:item",
                        "name": "minecraft:white_wool",
                        "condition": {
                            "type": "minecraft:entity_properties",
                            "entity": "this",
                            "predicate": {
                                "minecraft:components": { "minecraft:sheep/color": "white" }
                            }
                        }
                    }
                ]
            }"#,
        )
        .expect("alternatives entry should deserialize");

        let mut entries = Vec::new();
        let mut empty_weight = 0;
        extract_entries(&entry, LootCondition::None, &mut entries, &mut empty_weight);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].item, "minecraft:white_wool");
        assert_eq!(
            entries[0].condition,
            LootCondition::AllOf(&[LootCondition::KilledByPlayer, SHEEP_IS_WHITE])
        );
    }
}
