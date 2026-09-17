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

/// Decode an explicitly present filter and normalize registry names; null and malformed names are errors.
fn deserialize_component_filter<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Vec::<String>::deserialize(deserializer)?
        .into_iter()
        .map(|name| {
            Identifier::parse(&name)
                .map(|identifier| identifier.to_string())
                .map_err(serde::de::Error::custom)
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

/// Reject predicates that would otherwise silently turn component copying into an unconditional function.
fn validate_component_conditions(conditions: &[ConditionStruct]) -> Result<(), String> {
    for condition in conditions {
        let valid = match condition.condition.as_str() {
            "minecraft:survives_explosion" | "minecraft:killed_by_player" => true,
            "minecraft:random_chance" => condition.chance.is_some(),
            "minecraft:random_chance_with_enchanted_bonus" => {
                condition.unenchanted_chance.is_some() && condition.enchanted_chance.is_some()
            }
            "minecraft:table_bonus" => condition
                .chances
                .as_ref()
                .is_some_and(|list| !list.is_empty()),
            "minecraft:match_tool" => parse_condition(condition) != LootCondition::None,
            "minecraft:all_of" | "minecraft:any_of" => {
                if let Some(terms) = &condition.terms {
                    validate_component_conditions(terms)?;
                    condition.condition == "minecraft:all_of"
                        || (!terms.is_empty()
                            && terms.iter().all(|term| {
                                matches!(
                                    parse_condition(term),
                                    LootCondition::SilkTouch
                                        | LootCondition::Shears
                                        | LootCondition::SilkTouchOrShears
                                )
                            }))
                } else {
                    false
                }
            }
            "minecraft:inverted" => {
                if let Some(term) = &condition.term {
                    validate_component_conditions(std::slice::from_ref(term.as_ref()))?;
                    parse_condition(condition) != LootCondition::None
                } else {
                    false
                }
            }
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
