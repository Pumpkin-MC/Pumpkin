use std::{fs, path::Path};

use heck::ToShoutySnakeCase;
use proc_macro2::{Span, TokenStream};
use pumpkin_util::{
    identifier::Identifier,
    loot_table::{LootBonusFormula, LootCondition},
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
    remaining: serde_json::Map<String, serde_json::Value>,
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

#[derive(Deserialize, Clone, Debug)]
struct ConditionStruct {
    #[serde(default)]
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
    term: Option<Box<ConditionStruct>>,
    #[serde(default)]
    terms: Option<Vec<ConditionStruct>>,
    #[serde(flatten)]
    remaining: serde_json::Map<String, serde_json::Value>,
}

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
                    .any(|t| parse_condition(t) == LootCondition::SilkTouch);
                let has_shears = terms
                    .iter()
                    .any(|t| parse_condition(t) == LootCondition::Shears);
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
                match parse_condition(term) {
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

fn combine_conditions(conditions: &[ConditionStruct]) -> LootCondition {
    let mut parsed_list: Vec<LootCondition> = Vec::new();
    for c in conditions {
        let parsed = parse_condition(c);
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

#[derive(Deserialize, Clone, Debug)]
struct EntryFunctionJson {
    function: String,
    #[serde(default)]
    formula: Option<String>,
    #[serde(default)]
    parameters: Option<BonusParameterStruct>,
    count: Option<CountStruct>,
    #[serde(flatten)]
    remaining: serde_json::Map<String, serde_json::Value>,
}

#[derive(Deserialize, Clone, Debug)]
struct CopyComponentsStruct {
    source: String,
    #[serde(default, deserialize_with = "deserialize_component_filter")]
    include: Option<Vec<String>>,
    #[serde(default, deserialize_with = "deserialize_component_filter")]
    exclude: Option<Vec<String>>,
    #[serde(default)]
    conditions: Vec<ConditionStruct>,
}

/// Normalizes filter names and validates registry membership, rejecting null filters and asset-loading errors.
fn deserialize_component_filter<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Vec::<String>::deserialize(deserializer)?
        .into_iter()
        .map(|name| {
            let name = Identifier::parse(&name)
                .map_err(serde::de::Error::custom)?
                .to_string();
            if !crate::data_component::is_registered_component(&name)
                .map_err(serde::de::Error::custom)?
            {
                return Err(serde::de::Error::custom(format!(
                    "Unknown data component: {name}"
                )));
            }
            Ok(name)
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

#[derive(Deserialize, Clone, Debug)]
#[serde(try_from = "EntryFunctionJson")]
struct EntryFunctionStruct {
    function: String,
    formula: Option<String>,
    parameters: Option<BonusParameterStruct>,
    count: Option<CountStruct>,
    copy_components: Option<CopyComponentsStruct>,
}

impl TryFrom<EntryFunctionJson> for EntryFunctionStruct {
    type Error = String;

    /// Validate supported component functions while retaining the existing count and bonus fields.
    fn try_from(raw: EntryFunctionJson) -> Result<Self, Self::Error> {
        let copy_components = if raw.function == "minecraft:copy_components" {
            let copy: CopyComponentsStruct =
                serde_json::from_value(serde_json::Value::Object(raw.remaining))
                    .map_err(|error| format!("invalid copy_components function: {error}"))?;
            if copy.source != "block_entity" {
                return Err(format!(
                    "unsupported copy_components source: {}",
                    copy.source
                ));
            }
            validate_component_conditions(&copy.conditions)?;
            Some(copy)
        } else {
            None
        };
        Ok(Self {
            function: raw.function,
            formula: raw.formula,
            parameters: raw.parameters,
            count: raw.count,
            copy_components,
        })
    }
}

/// Reject condition details that the existing compact runtime representation cannot preserve.
fn validate_component_conditions(conditions: &[ConditionStruct]) -> Result<(), String> {
    for condition in conditions {
        let parameters = [
            condition.enchantment.is_some(),
            condition.chance.is_some(),
            condition.unenchanted_chance.is_some(),
            condition.enchanted_chance.is_some(),
            condition.chances.is_some(),
            condition.predicate.is_some(),
            condition.term.is_some(),
            condition.terms.is_some(),
        ]
        .into_iter()
        .filter(|present| *present)
        .count();
        let valid = condition.remaining.is_empty()
            && match condition.condition.as_str() {
                "minecraft:survives_explosion" | "minecraft:killed_by_player" => parameters == 0,
                "minecraft:random_chance" => parameters == 1 && condition.chance.is_some(),
                "minecraft:match_tool" => {
                    parameters == 1
                        && condition
                            .predicate
                            .as_ref()
                            .is_some_and(component_tool_predicate_is_supported)
                }
                "minecraft:all_of" | "minecraft:any_of" => {
                    if parameters == 1
                        && let Some(terms) = &condition.terms
                    {
                        validate_component_conditions(terms)?;
                        condition.condition == "minecraft:all_of"
                            || (!terms.is_empty()
                                && terms.iter().all(|term| {
                                    matches!(
                                        parse_condition(term),
                                        LootCondition::SilkTouch | LootCondition::Shears
                                    )
                                }))
                    } else {
                        false
                    }
                }
                "minecraft:inverted" => {
                    if parameters == 1
                        && let Some(term) = &condition.term
                    {
                        validate_component_conditions(std::slice::from_ref(term.as_ref()))?;
                        matches!(
                            parse_condition(term),
                            LootCondition::SilkTouch | LootCondition::SilkTouchOrShears
                        )
                    } else {
                        false
                    }
                }
                // Enchantment-dependent conditions discard the selected enchantment in the existing representation.
                _ => false,
            };
        if !valid {
            return Err(format!(
                "unsupported or malformed copy_components condition: {}",
                condition.condition
            ));
        }
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SilkTouchToolPredicate {
    #[serde(rename = "minecraft:enchantments")]
    enchantments: [ToolEnchantmentRequirement; 1],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ToolEnchantmentRequirement {
    enchantments: String,
    levels: MinimumToolEnchantmentLevel,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MinimumToolEnchantmentLevel {
    min: i32,
}

/// Accept only exact shears or positive silk-touch predicates represented by the existing runtime flags.
fn component_tool_predicate_is_supported(predicate: &PredicateStruct) -> bool {
    if !predicate.remaining.is_empty() {
        return false;
    }
    match (&predicate.items, &predicate.predicates) {
        (Some(serde_json::Value::String(item)), None) => {
            matches!(item.as_str(), "shears" | "minecraft:shears")
        }
        (Some(serde_json::Value::Array(items)), None) => {
            matches!(items.as_slice(), [serde_json::Value::String(item)] if matches!(item.as_str(), "shears" | "minecraft:shears"))
        }
        (None, Some(predicates)) => serde_json::from_value::<SilkTouchToolPredicate>(
            predicates.clone(),
        )
        .is_ok_and(|predicate| {
            let [requirement] = predicate.enchantments;
            matches!(
                requirement.enchantments.as_str(),
                "silk_touch" | "minecraft:silk_touch"
            ) && requirement.levels.min == 1
        }),
        _ => false,
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
enum LootTableValue {
    Reference(String),
    Inline(ChestLootTableJson),
}

#[derive(Deserialize, Clone, Debug)]
struct PoolEntryJson {
    #[serde(rename = "type")]
    entry_type: String,
    name: Option<String>,
    #[serde(default)]
    value: Option<LootTableValue>,
    #[serde(default = "default_weight")]
    weight: i32,
    #[serde(default)]
    functions: Vec<EntryFunctionStruct>,
    #[serde(default)]
    conditions: Vec<ConditionStruct>,
    #[serde(default)]
    children: Vec<PoolEntryStruct>,
}

/// An entry whose component functions fit the existing flattened loot representation.
#[derive(Deserialize, Clone, Debug)]
#[serde(try_from = "PoolEntryJson")]
struct PoolEntryStruct {
    entry_type: String,
    name: Option<String>,
    value: Option<LootTableValue>,
    weight: i32,
    functions: Vec<EntryFunctionStruct>,
    conditions: Vec<ConditionStruct>,
    children: Vec<PoolEntryStruct>,
}

impl TryFrom<PoolEntryJson> for PoolEntryStruct {
    type Error = String;

    /// Rejects lost wrapper functions and random copies mixed with separately evaluated count modifiers.
    fn try_from(entry: PoolEntryJson) -> Result<Self, Self::Error> {
        let has_count_modifier = entry.functions.iter().any(|function| {
            matches!(
                function.function.as_str(),
                "minecraft:set_count"
                    | "minecraft:apply_bonus"
                    | "minecraft:enchanted_count_increase"
            )
        });
        for copy in entry
            .functions
            .iter()
            .filter_map(|function| function.copy_components.as_ref())
        {
            if !matches!(
                entry.entry_type.as_str(),
                "minecraft:item" | "minecraft:tag"
            ) {
                return Err(format!(
                    "unsupported copy_components on {}: only item and tag entries preserve functions",
                    entry.entry_type
                ));
            }
            if has_count_modifier && copy.conditions.iter().any(component_condition_uses_random) {
                return Err(
                    "unsupported random copy_components conditions with count or bonus functions: their relative order is not represented"
                        .to_owned(),
                );
            }
        }
        Ok(Self {
            entry_type: entry.entry_type,
            name: entry.name,
            value: entry.value,
            weight: entry.weight,
            functions: entry.functions,
            conditions: entry.conditions,
            children: entry.children,
        })
    }
}

/// Detects RNG use in validated copy predicates, including predicates nested in conjunctions.
fn component_condition_uses_random(condition: &ConditionStruct) -> bool {
    matches!(
        condition.condition.as_str(),
        "minecraft:random_chance" | "minecraft:survives_explosion"
    ) || condition
        .terms
        .as_ref()
        .is_some_and(|terms| terms.iter().any(component_condition_uses_random))
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
    conditions: Vec<ConditionStruct>,
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
    functions: Vec<CopyComponentsStruct>,
}

/// Flatten supported entries while preserving item functions in their source order.
fn extract_entries(
    entry: &PoolEntryStruct,
    inherited_condition: LootCondition,
    out: &mut Vec<ParsedEntry>,
    empty_weight: &mut i32,
) {
    extract_entries_with_depth(entry, inherited_condition, out, empty_weight, 0);
}

/// Flatten nested entries within the existing depth limit without altering count or bonus extraction.
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

    let entry_cond = match (inherited_condition, combine_conditions(&entry.conditions)) {
        (LootCondition::None, cond) | (cond, LootCondition::None) => cond,
        (first, second) if first == second => first,
        (first, second) => LootCondition::AllOf(Box::leak(vec![first, second].into_boxed_slice())),
    };

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
                    functions: entry
                        .functions
                        .iter()
                        .filter_map(|function| function.copy_components.clone())
                        .collect(),
                });
            }
        }
        "minecraft:tag" => {
            let tag_name_opt = entry.name.as_deref().or_else(|| match &entry.value {
                Some(LootTableValue::Reference(r)) => Some(r.as_str()),
                _ => None,
            });
            if let Some(tag_name) = tag_name_opt {
                let tag_rel = tag_name.strip_prefix("minecraft:").unwrap_or(tag_name);
                let tag_path = Path::new("../../assets/datapacks/26_2/data/minecraft/tags/item")
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
                                functions: entry
                                    .functions
                                    .iter()
                                    .filter_map(|function| function.copy_components.clone())
                                    .collect(),
                            });
                        }
                    }
                }
            }
        }
        "minecraft:loot_table" => match &entry.value {
            Some(LootTableValue::Reference(table_name)) => {
                let table_rel = table_name.strip_prefix("minecraft:").unwrap_or(table_name);
                let table_path = Path::new("../../assets/datapacks/26_2/data/minecraft/loot_table")
                    .join(format!("{table_rel}.json"));
                if let Ok(content) = fs::read_to_string(&table_path) {
                    if let Ok(nested_table) = serde_json::from_str::<ChestLootTableJson>(&content) {
                        for pool in &nested_table.pools {
                            let mut pool_cond = entry_cond;
                            for c in &pool.conditions {
                                let parsed = parse_condition(c);
                                if parsed != LootCondition::None {
                                    pool_cond = parsed;
                                }
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
                    for c in &pool.conditions {
                        let parsed = parse_condition(c);
                        if parsed != LootCondition::None {
                            pool_cond = parsed;
                        }
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
                        Path::new("../../assets/datapacks/26_2/data/minecraft/loot_table")
                            .join(format!("{table_rel}.json"));
                    if let Ok(content) = fs::read_to_string(&table_path) {
                        if let Ok(nested_table) =
                            serde_json::from_str::<ChestLootTableJson>(&content)
                        {
                            for pool in &nested_table.pools {
                                let mut pool_cond = entry_cond;
                                for c in &pool.conditions {
                                    let parsed = parse_condition(c);
                                    if parsed != LootCondition::None {
                                        pool_cond = parsed;
                                    }
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
                let child_cond = combine_conditions(&child.conditions);

                let effective_cond = if child_cond == LootCondition::SilkTouch {
                    saw_silk = true;
                    LootCondition::SilkTouch
                } else if child_cond == LootCondition::Shears {
                    saw_shears = true;
                    LootCondition::Shears
                } else if child_cond == LootCondition::SilkTouchOrShears {
                    saw_silk = true;
                    saw_shears = true;
                    LootCondition::SilkTouchOrShears
                } else if saw_silk && saw_shears {
                    LootCondition::NoSilkTouchOrShears
                } else if saw_silk {
                    LootCondition::NoSilkTouch
                } else if saw_shears {
                    LootCondition::NoSilkTouchOrShears
                } else {
                    entry_cond
                };

                extract_entries_with_depth(child, effective_cond, out, empty_weight, depth + 1);
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

/// Emit an optional registry-name filter without conflating omission and an empty list.
fn component_filter_to_tokens(filter: Option<&[String]>) -> TokenStream {
    match filter {
        None => quote! { None },
        Some(names) => {
            let names = names
                .iter()
                .map(|name| LitStr::new(name, Span::call_site()));
            quote! { Some(&[#(#names),*]) }
        }
    }
}

/// Emit the component source, filters, and conditions for one ordered function.
fn function_to_tokens(function: &CopyComponentsStruct) -> TokenStream {
    let source = LitStr::new(&function.source, Span::call_site());
    let include = component_filter_to_tokens(function.include.as_deref());
    let exclude = component_filter_to_tokens(function.exclude.as_deref());
    let condition = condition_to_tokens(combine_conditions(&function.conditions));
    quote! {
        LootFunction::CopyComponents {
            source: #source,
            include: #include,
            exclude: #exclude,
            condition: #condition,
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

        let pool_cond = combine_conditions(&pool.conditions);

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
                let functions = e.functions.iter().map(function_to_tokens);

                quote! {
                    LootEntry {
                        item: #name_lit,
                        weight: #weight,
                        min_count: #min_count,
                        max_count: #max_count,
                        condition: #cond_tokens,
                        bonus_formula: #bonus_tokens,
                        functions: &[#(#functions),*],
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
                .to_string_lossy()
                .to_string();

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

/// Generate static loot definitions and registry lookup functions from the bundled tables.
pub fn build() -> TokenStream {
    let base = Path::new("../../assets/datapacks/26_2/data/minecraft/loot_table");

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
        /// Return a bundled loot table by its namespaced or short registry path.
        #[must_use]
        pub fn get_loot_table(key: &str) -> Option<&'static LootTable> {
            match key {
                #(#table_keys | #short_table_keys => Some(&#table_idents),)*
                _ => None,
            }
        }

        /// Return a bundled loot table through the legacy chest-loot lookup name.
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
    use super::*;

    /// Component copying must survive generation instead of being discarded from the bundled shulker table.
    #[test]
    fn bundled_shulker_retains_component_function() -> Result<(), Box<dyn std::error::Error>> {
        let table: ChestLootTableJson = serde_json::from_str(include_str!(
            "../../../assets/datapacks/26_2/data/minecraft/loot_table/blocks/shulker_box.json"
        ))?;
        let mut tokens = TokenStream::new();
        emit_table("SHULKER_BOX", &table, &mut tokens);
        let generated = tokens.to_string();
        assert!(generated.contains("CopyComponents"), "{generated}");
        assert!(generated.contains("minecraft:container"), "{generated}");
        Ok(())
    }

    /// A supported function with a missing source must fail parsing rather than disappear.
    #[test]
    fn copy_components_requires_a_source() {
        let parsed = serde_json::from_str::<EntryFunctionStruct>(
            r#"{"function":"minecraft:copy_components","include":["minecraft:custom_name"]}"#,
        );
        assert!(parsed.is_err());
    }

    /// Omitted filters include all values, while an explicit empty list must survive generation.
    #[test]
    fn copy_components_distinguishes_absent_and_empty_filters()
    -> Result<(), Box<dyn std::error::Error>> {
        let unrestricted: EntryFunctionStruct = serde_json::from_str(
            r#"{"function":"minecraft:copy_components","source":"block_entity"}"#,
        )?;
        let empty: EntryFunctionStruct = serde_json::from_str(
            r#"{"function":"minecraft:copy_components","source":"block_entity","include":[],"exclude":[]}"#,
        )?;
        let unrestricted = unrestricted
            .copy_components
            .ok_or("missing unrestricted function")?;
        let empty = empty.copy_components.ok_or("missing empty function")?;
        assert_eq!(unrestricted.include, None);
        assert_eq!(unrestricted.exclude, None);
        assert_eq!(empty.include, Some(Vec::new()));
        assert_eq!(empty.exclude, Some(Vec::new()));
        assert_eq!(
            component_filter_to_tokens(empty.include.as_deref()).to_string(),
            "Some (& [])"
        );
        assert_eq!(
            component_filter_to_tokens(unrestricted.include.as_deref()).to_string(),
            "None"
        );
        Ok(())
    }

    /// Filters may overlap, and their names and conditions must be retained without selecting values during generation.
    #[test]
    fn copy_components_retains_filters_and_conditions() -> Result<(), Box<dyn std::error::Error>> {
        let function: EntryFunctionStruct = serde_json::from_str(
            r#"{
                "function":"minecraft:copy_components",
                "source":"block_entity",
                "include":["custom_name","minecraft:container"],
                "exclude":["minecraft:container"],
                "conditions":[
                    {"condition":"minecraft:random_chance","chance":0.25},
                    {"condition":"minecraft:killed_by_player"}
                ]
            }"#,
        )?;
        let copy = function
            .copy_components
            .ok_or("missing component function")?;
        assert_eq!(
            copy.include.as_deref(),
            Some(
                [
                    "minecraft:custom_name".to_owned(),
                    "minecraft:container".to_owned()
                ]
                .as_slice()
            )
        );
        assert_eq!(
            copy.exclude.as_deref(),
            Some(["minecraft:container".to_owned()].as_slice())
        );
        assert_eq!(
            combine_conditions(&copy.conditions),
            LootCondition::AllOf(&[
                LootCondition::RandomChance { chance: 0.25 },
                LootCondition::KilledByPlayer,
            ]),
        );
        let generated = function_to_tokens(&copy).to_string();
        assert!(generated.contains("LootCondition :: AllOf"), "{generated}");
        assert!(generated.contains("0.25f32"), "{generated}");
        Ok(())
    }

    /// Invalid sources, filters, and unrepresented predicates must not silently change component-copy semantics.
    #[test]
    fn copy_components_rejects_malformed_definitions() {
        for definition in [
            r#"{"source":"this"}"#,
            r#"{"source":false}"#,
            r#"{"source":"block_entity","include":"minecraft:custom_name"}"#,
            r#"{"source":"block_entity","include":null}"#,
            r#"{"source":"block_entity","exclude":[2]}"#,
            r#"{"source":"block_entity","include":["Minecraft:CustomName"]}"#,
            r#"{"source":"block_entity","conditions":[{}]}"#,
            r#"{"source":"block_entity","conditions":[{"condition":"minecraft:random_chance"}]}"#,
            r#"{"source":"block_entity","conditions":[{"condition":"minecraft:all_of","terms":[{"condition":"unknown"}]}]}"#,
            r#"{"source":"block_entity","conditions":[{"condition":"minecraft:any_of","terms":[{"condition":"minecraft:random_chance","chance":0.5}]}]}"#,
        ] {
            let json = format!(
                "{{\"function\":\"minecraft:copy_components\",{}",
                &definition[1..]
            );
            assert!(
                serde_json::from_str::<EntryFunctionStruct>(&json).is_err(),
                "{json}"
            );
        }
    }

    /// Both filters reject names absent from the authoritative component registry.
    #[test]
    fn copy_components_rejects_unknown_component_ids() {
        for filter in ["include", "exclude"] {
            for name in ["minecraft:not_a_real_component", "example:custom_name"] {
                let mut definition = serde_json::json!({
                    "function": "minecraft:copy_components",
                    "source": "block_entity"
                });
                definition[filter] = serde_json::json!([name]);
                let error = serde_json::from_value::<EntryFunctionStruct>(definition)
                    .err()
                    .map(|error| error.to_string());
                assert!(
                    error.is_some_and(|error| error.contains("Unknown data component")),
                    "{filter} accepted unknown component {name}"
                );
            }
        }
    }

    /// Wrapper copies are rejected before flattening can discard their scope or ordering.
    #[test]
    fn nested_loot_table_wrapper_rejects_component_functions() {
        for value in [
            serde_json::json!("minecraft:blocks/chest"),
            serde_json::json!({"pools": [{"rolls": 1, "entries": [{
                "type": "minecraft:item", "name": "minecraft:diamond"
            }]}]}),
        ] {
            let definition = serde_json::json!({"pools": [{"rolls": 1, "entries": [{
                "type": "minecraft:loot_table",
                "value": value,
                "functions": [{"function": "minecraft:copy_components", "source": "block_entity"}]
            }]}]});
            let error = serde_json::from_value::<ChestLootTableJson>(definition)
                .err()
                .map(|error| error.to_string());
            assert!(
                error
                    .is_some_and(|error| error.contains("copy_components on minecraft:loot_table")),
                "nested wrapper copy must fail before generation"
            );
        }
    }

    /// Random copy predicates cannot be combined with count modifiers whose order is not represented.
    #[test]
    fn random_component_copies_reject_count_and_bonus_combinations() {
        let modifiers = [
            serde_json::json!({"function": "minecraft:set_count", "count": {"type": "minecraft:uniform", "min": 1, "max": 2}}),
            serde_json::json!({"function": "minecraft:set_count", "count": 0}),
            serde_json::json!({"function": "minecraft:apply_bonus", "formula": "minecraft:ore_drops"}),
            serde_json::json!({"function": "minecraft:enchanted_count_increase", "count": 1}),
        ];
        let conditions = [
            serde_json::json!({"condition": "minecraft:random_chance", "chance": 0.5}),
            serde_json::json!({"condition": "minecraft:survives_explosion"}),
            serde_json::json!({"condition": "minecraft:all_of", "terms": [
                {"condition": "minecraft:killed_by_player"},
                {"condition": "minecraft:random_chance", "chance": 0.5}
            ]}),
        ];
        for modifier in &modifiers {
            for condition in &conditions {
                let copy = serde_json::json!({
                    "function": "minecraft:copy_components", "source": "block_entity", "conditions": [condition]
                });
                for functions in [
                    serde_json::json!([copy, modifier]),
                    serde_json::json!([modifier, copy]),
                ] {
                    let definition = serde_json::json!({
                        "type": "minecraft:item", "name": "minecraft:chest", "functions": functions
                    });
                    let error = serde_json::from_value::<PoolEntryStruct>(definition)
                        .err()
                        .map(|error| error.to_string());
                    assert!(
                        error.is_some_and(|error| error.contains(
                            "random copy_components conditions with count or bonus functions"
                        )),
                        "count/copy combinations must fail instead of losing function order"
                    );
                }
            }
        }
    }

    /// Independent random copies and deterministic copies with count modifiers remain supported.
    #[test]
    fn representable_component_function_combinations_remain_supported()
    -> Result<(), Box<dyn std::error::Error>> {
        for functions in [
            serde_json::json!([{"function": "minecraft:copy_components", "source": "block_entity", "conditions": [
                {"condition": "minecraft:random_chance", "chance": 0.5}
            ]}]),
            serde_json::json!([
                {"function": "minecraft:copy_components", "source": "block_entity", "conditions": [
                    {"condition": "minecraft:killed_by_player"}
                ]},
                {"function": "minecraft:set_count", "count": {"type": "minecraft:uniform", "min": 1, "max": 2}}
            ]),
        ] {
            let _: PoolEntryStruct = serde_json::from_value(serde_json::json!({
                "type": "minecraft:item", "name": "minecraft:chest", "functions": functions
            }))?;
        }
        let _: ChestLootTableJson =
            serde_json::from_value(serde_json::json!({"pools": [{"rolls": 1, "entries": [{
                "type": "minecraft:loot_table", "value": {"pools": [{"rolls": 1, "entries": [{
                    "type": "minecraft:item", "name": "minecraft:chest", "functions": [{
                        "function": "minecraft:copy_components", "source": "block_entity"
                    }]
                }]}]}
            }]}]}))?;
        Ok(())
    }

    /// Component functions reject predicates whose extra requirements the compact runtime condition would lose.
    #[test]
    fn copy_components_rejects_unrepresented_condition_details()
    -> Result<(), Box<dyn std::error::Error>> {
        for condition in [
            r#"{"condition":"minecraft:match_tool","predicate":{"items":"minecraft:shears","count":2}}"#,
            r#"{"condition":"minecraft:match_tool","predicate":{"items":["minecraft:shears","minecraft:stick"]}}"#,
            r#"{"condition":"minecraft:match_tool","predicate":{"items":"minecraft:not_shears"}}"#,
            r#"{"condition":"minecraft:match_tool","predicate":{"predicates":{"minecraft:enchantments":[{"enchantments":"minecraft:silk_touch","levels":{"min":2}}]}}}"#,
            r#"{"condition":"minecraft:match_tool","predicate":{"predicates":{"minecraft:enchantments":[{"enchantments":"minecraft:silk_touch","levels":{"min":1,"max":1}}]}}}"#,
            r#"{"condition":"minecraft:match_tool","predicate":{"predicates":{"unsupported":"minecraft:silk_touch"}}}"#,
            r#"{"condition":"minecraft:inverted","term":{"condition":"minecraft:match_tool","predicate":{"items":"minecraft:shears"}}}"#,
            r#"{"condition":"minecraft:any_of","terms":[{"condition":"minecraft:any_of","terms":[{"condition":"minecraft:match_tool","predicate":{"items":"minecraft:shears"}},{"condition":"minecraft:match_tool","predicate":{"predicates":{"minecraft:enchantments":[{"enchantments":"minecraft:silk_touch","levels":{"min":1}}]}}}]},{"condition":"minecraft:match_tool","predicate":{"predicates":{"minecraft:enchantments":[{"enchantments":"minecraft:silk_touch","levels":{"min":1}}]}}}]}"#,
            r#"{"condition":"minecraft:table_bonus","enchantment":"minecraft:silk_touch","chances":[0.0,1.0]}"#,
            r#"{"condition":"minecraft:random_chance_with_enchanted_bonus","enchantment":"minecraft:fortune","unenchanted_chance":0.0,"enchanted_chance":1.0}"#,
            r#"{"condition":"minecraft:random_chance","chance":0.5,"predicate":{"items":"minecraft:shears"}}"#,
            r#"{"condition":"minecraft:killed_by_player","unsupported":true}"#,
        ] {
            let function = serde_json::json!({
                "function": "minecraft:copy_components",
                "source": "block_entity",
                "conditions": [serde_json::from_str::<serde_json::Value>(condition)?],
            });
            assert!(
                serde_json::from_value::<EntryFunctionStruct>(function).is_err(),
                "{condition}"
            );
        }
        Ok(())
    }

    /// Exact supported predicates retain their meaning when combined, repeated, or inverted.
    #[test]
    fn copy_components_retains_supported_tool_conditions() -> Result<(), Box<dyn std::error::Error>>
    {
        let silk = serde_json::json!({
            "condition": "minecraft:match_tool",
            "predicate": { "predicates": { "minecraft:enchantments": [{
                "enchantments": "minecraft:silk_touch", "levels": { "min": 1 }
            }] } },
        });
        let shears = serde_json::json!({
            "condition": "minecraft:match_tool",
            "predicate": { "items": "minecraft:shears" },
        });
        let either = serde_json::json!({
            "condition": "minecraft:any_of", "terms": [silk.clone(), shears.clone()],
        });
        for (condition, expected) in [
            (silk.clone(), LootCondition::SilkTouch),
            (shears.clone(), LootCondition::Shears),
            (either.clone(), LootCondition::SilkTouchOrShears),
            (
                serde_json::json!({ "condition": "minecraft:any_of", "terms": [silk.clone(), silk.clone()] }),
                LootCondition::SilkTouch,
            ),
            (
                serde_json::json!({ "condition": "minecraft:any_of", "terms": [shears.clone(), shears] }),
                LootCondition::Shears,
            ),
            (
                serde_json::json!({ "condition": "minecraft:inverted", "term": silk }),
                LootCondition::NoSilkTouch,
            ),
            (
                serde_json::json!({ "condition": "minecraft:inverted", "term": either }),
                LootCondition::NoSilkTouchOrShears,
            ),
        ] {
            let function: EntryFunctionStruct = serde_json::from_value(serde_json::json!({
                "function": "minecraft:copy_components", "source": "block_entity", "conditions": [condition],
            }))?;
            let copy = function
                .copy_components
                .ok_or("missing component function")?;
            assert_eq!(combine_conditions(&copy.conditions), expected);
        }
        Ok(())
    }

    /// Ordered component copies do not replace the established count, weight, and bonus extraction.
    #[test]
    fn copying_preserves_order_and_legacy_count_fields() -> Result<(), Box<dyn std::error::Error>> {
        let source = r#"{
            "type":"minecraft:item","name":"minecraft:diamond","weight":3,
            "functions":[
                {"function":"minecraft:copy_components","source":"block_entity","include":["minecraft:custom_name"]},
                {"function":"minecraft:set_count","count":{"type":"minecraft:uniform","min":2,"max":4}},
                {"function":"minecraft:apply_bonus","formula":"minecraft:uniform_bonus_count","parameters":{"bonusMultiplier":2}},
                {"function":"minecraft:copy_components","source":"block_entity","include":["minecraft:container"]}
            ]
        }"#;
        let mut entry: PoolEntryStruct = serde_json::from_str(source)?;
        let mut with_copy = Vec::new();
        let mut empty_weight = 0;
        extract_entries(
            &entry,
            LootCondition::None,
            &mut with_copy,
            &mut empty_weight,
        );
        entry
            .functions
            .retain(|function| function.copy_components.is_none());
        let mut without_copy = Vec::new();
        extract_entries(
            &entry,
            LootCondition::None,
            &mut without_copy,
            &mut empty_weight,
        );
        let ([with_copy], [without_copy]) = (with_copy.as_slice(), without_copy.as_slice()) else {
            return Err("expected one item from each table".into());
        };
        assert_eq!(
            (with_copy.weight, with_copy.min_count, with_copy.max_count),
            (3, 2, 4)
        );
        assert_eq!(
            (with_copy.weight, with_copy.min_count, with_copy.max_count),
            (
                without_copy.weight,
                without_copy.min_count,
                without_copy.max_count
            )
        );
        assert_eq!(
            with_copy.bonus_formula,
            Some(LootBonusFormula::UniformBonusCount(2))
        );
        assert_eq!(with_copy.bonus_formula, without_copy.bonus_formula);
        assert!(without_copy.functions.is_empty());
        let [first, second] = with_copy.functions.as_slice() else {
            return Err("expected two ordered component functions".into());
        };
        assert_eq!(
            first.include.as_deref(),
            Some(["minecraft:custom_name".to_owned()].as_slice())
        );
        assert_eq!(
            second.include.as_deref(),
            Some(["minecraft:container".to_owned()].as_slice())
        );
        Ok(())
    }

    /// Every bundled component-copying block table must retain its supported function after generation.
    #[test]
    fn bundled_component_tables_generate_without_loss() -> Result<(), Box<dyn std::error::Error>> {
        let base = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/datapacks/26_2/data/minecraft/loot_table/blocks");
        let mut component_tables = 0;
        for entry in fs::read_dir(base)? {
            let entry = entry?;
            let content = fs::read_to_string(entry.path())?;
            if !content.contains("minecraft:copy_components") {
                continue;
            }
            component_tables += 1;
            let table: ChestLootTableJson = serde_json::from_str(&content)?;
            let mut generated = TokenStream::new();
            emit_table("BLOCK", &table, &mut generated);
            assert!(
                generated.to_string().contains("CopyComponents"),
                "{}",
                entry.path().display()
            );
        }
        assert_eq!(component_tables, 71);
        Ok(())
    }
}
