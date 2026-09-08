use std::{collections::BTreeMap, fs, path::Path};

use heck::ToShoutySnakeCase;
use proc_macro2::{Span, TokenStream};
use pumpkin_util::loot_table::{LootBonusFormula, LootCondition};
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
    #[serde(default)]
    block: Option<String>,
    #[serde(default)]
    properties: Option<BTreeMap<String, String>>,
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
        "minecraft:block_state_property" => match &cond.block {
            Some(block) if !block.is_empty() => {
                let block: &'static str = Box::leak(block.clone().into_boxed_str());
                let mut properties: Vec<(&'static str, &'static str)> = Vec::new();
                if let Some(props) = &cond.properties {
                    for (key, value) in props {
                        properties.push((
                            Box::leak(key.clone().into_boxed_str()),
                            Box::leak(value.clone().into_boxed_str()),
                        ));
                    }
                }
                LootCondition::BlockStateProperty {
                    block,
                    properties: Box::leak(properties.into_boxed_slice()),
                }
            }
            _ => LootCondition::None,
        },
        _ => LootCondition::None,
    }
}

fn combine_conditions(conditions: &[ConditionStruct]) -> LootCondition {
    merge_conditions(conditions.iter().map(parse_condition).collect())
}

/// Combines parsed conditions, dropping `None`s.
/// Returns a single condition when possible, otherwise `AllOf`.
///
/// Repeated stochastic conditions (e.g. two identical `random_chance` terms in an
/// `all_of`) must each roll, so equal conditions are intentionally kept.
fn merge_conditions(conditions: Vec<LootCondition>) -> LootCondition {
    let mut kept: Vec<LootCondition> = Vec::with_capacity(conditions.len());
    for cond in conditions {
        if cond != LootCondition::None {
            kept.push(cond);
        }
    }
    match kept.len() {
        0 => LootCondition::None,
        1 => kept[0],
        _ => LootCondition::AllOf(Box::leak(kept.into_boxed_slice())),
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
struct EntryFunctionStruct {
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
    functions: Vec<EntryFunctionStruct>,
    #[serde(default)]
    conditions: Vec<ConditionStruct>,
}

fn default_rolls() -> RollsStruct {
    RollsStruct::Constant(1.0)
}

#[derive(Deserialize, Clone, Debug)]
struct ChestLootTableJson {
    #[serde(default)]
    functions: Vec<EntryFunctionStruct>,
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

/// Resolves count and bonus functions across the entry, pool, and table
/// function layers. The first layer providing a value wins.
fn resolve_functions(
    function_layers: &[&[EntryFunctionStruct]],
) -> (i32, i32, Option<LootBonusFormula>) {
    let (min_count, max_count) = function_layers
        .iter()
        .find_map(|fns| fns.iter().find(|f| f.function == "minecraft:set_count"))
        .and_then(|f| f.count.as_ref())
        .map(|c| (c.min(), c.max()))
        .unwrap_or((1, 1));

    let bonus_formula = function_layers.iter().find_map(|fns| {
        fns.iter().find_map(|f| {
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
                        let extra = f.parameters.as_ref().and_then(|p| p.extra).unwrap_or(0);
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
        })
    });

    (min_count, max_count, bonus_formula)
}

fn extract_entries(
    entry: &PoolEntryStruct,
    inherited_condition: LootCondition,
    pool_functions: &[EntryFunctionStruct],
    table_functions: &[EntryFunctionStruct],
    out: &mut Vec<ParsedEntry>,
    empty_weight: &mut i32,
) {
    extract_entries_with_depth(
        entry,
        inherited_condition,
        pool_functions,
        table_functions,
        out,
        empty_weight,
        0,
    );
}

fn extract_entries_with_depth(
    entry: &PoolEntryStruct,
    inherited_condition: LootCondition,
    pool_functions: &[EntryFunctionStruct],
    table_functions: &[EntryFunctionStruct],
    out: &mut Vec<ParsedEntry>,
    empty_weight: &mut i32,
    depth: usize,
) {
    if depth > 5 {
        return;
    }

    let entry_cond = match (inherited_condition, combine_conditions(&entry.conditions)) {
        (LootCondition::None, cond) | (cond, LootCondition::None) => cond,
        // Equal conditions are combined, not collapsed: a repeated stochastic
        // condition (e.g. two `random_chance` terms) must roll independently.
        (first, second) => LootCondition::AllOf(Box::leak(vec![first, second].into_boxed_slice())),
    };

    match entry.entry_type.as_str() {
        "minecraft:empty" => {
            *empty_weight += entry.weight;
        }
        "minecraft:item" => {
            if let Some(name) = &entry.name {
                let (min_count, max_count, bonus_formula) = resolve_functions(&[
                    entry.functions.as_slice(),
                    pool_functions,
                    table_functions,
                ]);

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
                        let (min_count, max_count, bonus_formula) = resolve_functions(&[
                            entry.functions.as_slice(),
                            pool_functions,
                            table_functions,
                        ]);
                        for item_name in tag_data.values {
                            out.push(ParsedEntry {
                                item: item_name,
                                weight: entry.weight,
                                min_count,
                                max_count,
                                condition: entry_cond,
                                bonus_formula,
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
                            let pool_cond = {
                                let mut conds = vec![entry_cond];
                                for c in &pool.conditions {
                                    conds.push(parse_condition(c));
                                }
                                merge_conditions(conds)
                            };
                            for child_entry in &pool.entries {
                                extract_entries_with_depth(
                                    child_entry,
                                    pool_cond,
                                    &pool.functions,
                                    &nested_table.functions,
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
                    let pool_cond = {
                        let mut conds = vec![entry_cond];
                        for c in &pool.conditions {
                            conds.push(parse_condition(c));
                        }
                        merge_conditions(conds)
                    };
                    for child_entry in &pool.entries {
                        extract_entries_with_depth(
                            child_entry,
                            pool_cond,
                            &pool.functions,
                            &nested_table.functions,
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
                                let pool_cond = {
                                    let mut conds = vec![entry_cond];
                                    for c in &pool.conditions {
                                        conds.push(parse_condition(c));
                                    }
                                    merge_conditions(conds)
                                };
                                for child_entry in &pool.entries {
                                    extract_entries_with_depth(
                                        child_entry,
                                        pool_cond,
                                        &pool.functions,
                                        &nested_table.functions,
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

                extract_entries_with_depth(
                    child,
                    effective_cond,
                    pool_functions,
                    table_functions,
                    out,
                    empty_weight,
                    depth + 1,
                );
            }
        }
        "minecraft:sequence" | "minecraft:group" => {
            for child in &entry.children {
                extract_entries_with_depth(
                    child,
                    entry_cond,
                    pool_functions,
                    table_functions,
                    out,
                    empty_weight,
                    depth + 1,
                );
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
        LootCondition::BlockStateProperty { block, properties } => {
            let block_lit = syn::LitStr::new(block, proc_macro2::Span::call_site());
            let props: Vec<TokenStream> = properties
                .iter()
                .map(|(key, value)| {
                    let key = syn::LitStr::new(key, proc_macro2::Span::call_site());
                    let value = syn::LitStr::new(value, proc_macro2::Span::call_site());
                    quote! { (#key, #value) }
                })
                .collect();
            quote! {
                LootCondition::BlockStateProperty {
                    block: #block_lit,
                    properties: &[#(#props),*],
                }
            }
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

        let pool_cond = combine_conditions(&pool.conditions);

        let mut parsed_entries = Vec::new();
        let mut empty_weight: i32 = 0;

        for entry in &pool.entries {
            extract_entries(
                entry,
                LootCondition::None,
                &pool.functions,
                &table.functions,
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
                .to_string_lossy()
                .replace('\\', "/");

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

/// Read every loot JSON from `../../assets/datapacks/26_2/data/minecraft/loot_table/` (recursively)
/// and emit a `pumpkin-data/src/generated/chest_loot.rs` with static constants
/// and a `get_chest_loot_table(key) -> Option<&'static ChestLootTable>` function.
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
