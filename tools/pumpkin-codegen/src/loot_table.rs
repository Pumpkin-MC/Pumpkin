use std::{fs, path::Path};

use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use pumpkin_util::loot_table::{
    DynamicLootCondition, DynamicLootEntry, DynamicLootEntryKind, DynamicLootFunction,
    DynamicLootPool, DynamicLootTable, LootBonusFormula, LootEntityTarget, LootFunctionKind,
    LootNumberProvider, parser::parse_loot_table_with_predicates,
};
use quote::{format_ident, quote};
use serde::Deserialize;
use serde_json::Value;

pub fn one_or_many<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
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

#[derive(Default)]
struct Generator {
    predicates: TokenStream,
    predicate_count: usize,
}

fn json_tokens(value: &Value) -> TokenStream {
    match value {
        Value::Null => quote! { LootPredicateValue::Null },
        Value::Bool(value) => quote! { LootPredicateValue::Bool(#value) },
        Value::Number(value) => {
            if let Some(value) = value.as_i64() {
                quote! { LootPredicateValue::from(#value) }
            } else if let Some(value) = value.as_u64() {
                quote! { LootPredicateValue::from(#value) }
            } else {
                let value = value.as_f64().unwrap();
                quote! { LootPredicateValue::from(#value) }
            }
        }
        Value::String(value) => quote! { LootPredicateValue::String(#value.to_owned()) },
        Value::Array(values) => {
            let values = values.iter().map(json_tokens);
            quote! { LootPredicateValue::Array(vec![#(#values),*]) }
        }
        Value::Object(values) => {
            let values = values.iter().map(|(key, value)| {
                let value = json_tokens(value);
                quote! { (#key.to_owned(), #value) }
            });
            quote! { LootPredicateValue::Object([#(#values),*].into_iter().collect()) }
        }
    }
}

fn number_tokens(provider: LootNumberProvider) -> TokenStream {
    match provider {
        LootNumberProvider::Constant(value) => quote! { LootNumberProvider::Constant(#value) },
        LootNumberProvider::Uniform { min, max } => {
            quote! {LootNumberProvider::Uniform {min: #min, max: #max}}
        }
        LootNumberProvider::Binomial { n, probability } => {
            quote! {LootNumberProvider::Binomial {n: #n, probability: #probability}}
        }
    }
}

fn optional_number_tokens(value: Option<LootNumberProvider>) -> TokenStream {
    value.map_or_else(
        || quote! {None},
        |value| {
            let value = number_tokens(value);
            quote! {Some(#value)}
        },
    )
}

fn bonus_tokens(formula: LootBonusFormula) -> TokenStream {
    match formula {
        LootBonusFormula::OreDrops => quote! { LootBonusFormula::OreDrops },
        LootBonusFormula::UniformBonusCount(multiplier) => {
            quote! { LootBonusFormula::UniformBonusCount(#multiplier) }
        }
        LootBonusFormula::BinomialWithBonusCount { extra, probability } => {
            quote! { LootBonusFormula::BinomialWithBonusCount { extra: #extra, probability: #probability } }
        }
    }
}

impl Generator {
    fn predicate(&mut self, predicate: &Value) -> TokenStream {
        let name = format_ident!("LOOT_PREDICATE_{}", self.predicate_count);
        self.predicate_count += 1;
        let predicate = json_tokens(predicate);
        self.predicates.extend(quote! {static #name: std::sync::LazyLock<LootPredicateValue> = std::sync::LazyLock::new(|| #predicate);});
        quote! { &#name }
    }

    fn condition(&mut self, condition: &DynamicLootCondition) -> TokenStream {
        match condition {
            DynamicLootCondition::None => quote! { LootCondition::None },
            DynamicLootCondition::Never => quote! { LootCondition::Never },
            DynamicLootCondition::SilkTouch => quote! { LootCondition::SilkTouch },
            DynamicLootCondition::NoSilkTouch => quote! { LootCondition::NoSilkTouch },
            DynamicLootCondition::Shears => quote! { LootCondition::Shears },
            DynamicLootCondition::SilkTouchOrShears => quote! { LootCondition::SilkTouchOrShears },
            DynamicLootCondition::NoSilkTouchOrShears => {
                quote! { LootCondition::NoSilkTouchOrShears }
            }
            DynamicLootCondition::SurvivesExplosion => quote! { LootCondition::SurvivesExplosion },
            DynamicLootCondition::KilledByPlayer => quote! { LootCondition::KilledByPlayer },
            DynamicLootCondition::RandomChance { chance } => {
                quote! { LootCondition::RandomChance { chance: #chance } }
            }
            DynamicLootCondition::RandomChanceWithEnchantedBonus {
                enchantment,
                unenchanted_chance,
                enchanted_chance_base,
                enchanted_chance_per_level_above_first,
            } => {
                quote! { LootCondition::RandomChanceWithEnchantedBonus { enchantment: #enchantment, unenchanted_chance: #unenchanted_chance, enchanted_chance_base: #enchanted_chance_base, enchanted_chance_per_level_above_first: #enchanted_chance_per_level_above_first } }
            }
            DynamicLootCondition::TableBonus {
                enchantment,
                chances,
            } => {
                quote! { LootCondition::TableBonus { enchantment: #enchantment, chances: &[#(#chances),*] } }
            }
            DynamicLootCondition::AllOf(conditions) => {
                let conditions: Vec<_> = conditions
                    .iter()
                    .map(|condition| self.condition(condition))
                    .collect();
                quote! { LootCondition::AllOf(&[#(#conditions),*]) }
            }
            DynamicLootCondition::AnyOf(conditions) => {
                let conditions: Vec<_> = conditions
                    .iter()
                    .map(|condition| self.condition(condition))
                    .collect();
                quote! { LootCondition::AnyOf(&[#(#conditions),*]) }
            }
            DynamicLootCondition::Inverted(condition) => {
                let condition = self.condition(condition);
                quote! { LootCondition::Inverted(&#condition) }
            }
            DynamicLootCondition::EntityOnFire => quote! { LootCondition::EntityOnFire },
            DynamicLootCondition::DirectAttackerWeaponEnchantment(tag) => {
                quote! { LootCondition::DirectAttackerWeaponEnchantment(#tag) }
            }
            DynamicLootCondition::EntityProperties { target, predicate } => {
                let target = match target {
                    LootEntityTarget::This => quote! { LootEntityTarget::This },
                    LootEntityTarget::Attacker => quote! { LootEntityTarget::Attacker },
                    LootEntityTarget::DirectAttacker => quote! { LootEntityTarget::DirectAttacker },
                };
                let predicate = self.predicate(predicate);
                quote! { LootCondition::EntityProperties { target: #target, predicate: #predicate } }
            }
            DynamicLootCondition::LocationCheck { offset, predicate } => {
                let predicate = self.predicate(predicate);
                quote! {LootCondition::LocationCheck {offset: [#(#offset),*], predicate: #predicate}}
            }
            DynamicLootCondition::MatchBlock(predicate) => {
                let predicate = self.predicate(predicate);
                quote! {LootCondition::MatchBlock(#predicate)}
            }
            DynamicLootCondition::MatchTool(predicate) => {
                let predicate = self.predicate(predicate);
                quote! {LootCondition::MatchTool(#predicate)}
            }
            DynamicLootCondition::WeatherCheck {
                raining,
                thundering,
            } => {
                let option = |value: &Option<bool>| {
                    value.map_or_else(|| quote! {None}, |value| quote! {Some(#value)})
                };
                let raining = option(raining);
                let thundering = option(thundering);
                quote! {LootCondition::WeatherCheck {raining: #raining, thundering: #thundering}}
            }
            DynamicLootCondition::DamageSourceTag { tag, expected } => {
                quote! { LootCondition::DamageSourceTag { tag: #tag, expected: #expected } }
            }
        }
    }

    fn functions(&mut self, functions: &[DynamicLootFunction]) -> TokenStream {
        let functions: Vec<_> = functions
            .iter()
            .map(|function| {
                let kind = match function.kind {
                    LootFunctionKind::SetCount { count, add } => {
                        let count = number_tokens(count);
                        quote! { LootFunctionKind::SetCount { count: #count, add: #add } }
                    }
                    LootFunctionKind::EnchantedCountIncrease {count, limit} => {
                        let count = number_tokens(count);
                        quote! {LootFunctionKind::EnchantedCountIncrease {count: #count, limit: #limit}}
                    }
                    LootFunctionKind::LimitCount {min, max} => {
                        let min = optional_number_tokens(min); let max = optional_number_tokens(max);
                        quote! {LootFunctionKind::LimitCount {min: #min, max: #max}}
                    }
                    LootFunctionKind::ExplosionDecay => quote! {LootFunctionKind::ExplosionDecay},
                    LootFunctionKind::ApplyBonus(formula) => {
                        let formula = bonus_tokens(formula);
                        quote! { LootFunctionKind::ApplyBonus(#formula) }
                    }
                    LootFunctionKind::FurnaceSmelt => quote! { LootFunctionKind::FurnaceSmelt },
                };
                let condition = self.condition(&function.condition);
                let enchantment = function.enchantment.as_ref().map_or_else(|| quote! {None}, |name| quote! {Some(#name)});
                quote! { LootFunction { kind: #kind, enchantment: #enchantment, condition: #condition } }
            })
            .collect();
        quote! { &[#(#functions),*] }
    }

    fn entries(&mut self, entries: &[DynamicLootEntry]) -> TokenStream {
        let entries: Vec<_> = entries.iter().map(|entry| {
            let kind = match &entry.kind {
                DynamicLootEntryKind::Item(item) => quote! { LootEntryKind::Item(#item) },
                DynamicLootEntryKind::Dynamic(name) => quote! { LootEntryKind::Dynamic(#name) },
                DynamicLootEntryKind::Empty => quote! { LootEntryKind::Empty },
                DynamicLootEntryKind::Tag { name, expand } => quote! { LootEntryKind::Tag { name: #name, expand: #expand } },
                DynamicLootEntryKind::Table(key) => quote! { LootEntryKind::Table(#key) },
                DynamicLootEntryKind::InlineTable(table) => { let table = self.table(table); quote! { LootEntryKind::InlineTable(&#table) } }
                DynamicLootEntryKind::Group(children) => {let children = self.entries(children); quote! {LootEntryKind::Group(#children)}}
                DynamicLootEntryKind::Sequence(children) => {let children = self.entries(children); quote! {LootEntryKind::Sequence(#children)}}
                DynamicLootEntryKind::Alternatives(children) => { let children = self.entries(children); quote! { LootEntryKind::Alternatives(#children) } }
            };
            let weight = entry.weight;
            let quality = entry.quality;
            let condition = self.condition(&entry.condition);
            let functions = self.functions(&entry.functions);
            quote! { LootEntry { kind: #kind, weight: #weight, quality: #quality, condition: #condition, functions: #functions } }
        }).collect();
        quote! { &[#(#entries),*] }
    }

    fn pool(&mut self, pool: &DynamicLootPool) -> TokenStream {
        let entries = self.entries(&pool.entries);
        let rolls = number_tokens(pool.rolls);
        let bonus_rolls = number_tokens(pool.bonus_rolls);
        let condition = self.condition(&pool.condition);
        let functions = self.functions(&pool.functions);
        quote! { LootPool { entries: #entries, rolls: #rolls, bonus_rolls: #bonus_rolls, condition: #condition, functions: #functions } }
    }

    fn table(&mut self, table: &DynamicLootTable) -> TokenStream {
        let pools: Vec<_> = table.pools.iter().map(|pool| self.pool(pool)).collect();
        let functions = self.functions(&table.functions);
        quote! { LootTable { pools: &[#(#pools),*], functions: #functions } }
    }
}

fn collect_json_files(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_json_files(&path, files);
        } else if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            files.push(path);
        }
    }
}

fn collect_tables(base: &Path, dir: &Path, tables: &mut Vec<(String, DynamicLootTable)>) {
    for entry in fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .flatten()
    {
        let path = entry.path();
        if path.is_dir() {
            collect_tables(base, &path, tables);
        } else if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            let key = path
                .strip_prefix(base)
                .unwrap()
                .with_extension("")
                .components()
                .map(|component| component.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/");
            let json = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            let table = parse_loot_table_with_predicates(&json, &|name| {
                let name = name.strip_prefix("minecraft:").unwrap_or(name);
                let file = Path::new("../../assets/datapack/data/minecraft/predicate")
                    .join(format!("{name}.json"));
                serde_json::from_str(&fs::read_to_string(file).ok()?).ok()
            })
            .unwrap_or_else(|| panic!("invalid loot table {}", path.display()));
            tables.push((key, table));
        }
    }
}

pub fn build() -> TokenStream {
    let base = Path::new("../../assets/datapack/data/minecraft/loot_table");
    let mut tables = Vec::new();
    collect_tables(base, base, &mut tables);
    tables.sort_by(|a, b| a.0.cmp(&b.0));
    let mut generator = Generator::default();
    let mut declarations = TokenStream::new();
    let mut lookup = Vec::new();
    for (key, table) in &tables {
        let name = format_ident!("{}", key.replace('/', "_").to_shouty_snake_case());
        let table = generator.table(table);
        declarations.extend(quote! { pub static #name: LootTable = #table; });
        lookup.push((format!("minecraft:{key}"), name.clone()));
        lookup.push((key.clone(), name));
    }
    lookup.sort_by(|a, b| a.0.cmp(&b.0));
    let rows: Vec<_> = lookup
        .iter()
        .map(|(key, name)| quote! { (#key, &#name) })
        .collect();
    let count = rows.len();
    let predicate_base = Path::new("../../assets/datapack/data/minecraft/predicate");
    let mut predicate_rows = Vec::new();
    let mut predicate_files = Vec::new();
    collect_json_files(predicate_base, &mut predicate_files);
    predicate_files.sort();
    for path in &predicate_files {
        let key = path
            .strip_prefix(predicate_base)
            .unwrap()
            .with_extension("")
            .to_string_lossy()
            .replace('\\', "/");
        let key = format!("minecraft:{key}");
        let value: Value = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        let value = json_tokens(&value);
        let name = format_ident!("LOOT_NAMED_PREDICATE_{}", predicate_rows.len());
        declarations.extend(quote! { static #name: std::sync::LazyLock<LootPredicateValue> = std::sync::LazyLock::new(|| #value); });
        predicate_rows.push(quote! { #key => Some(&#name), });
    }
    let predicates = generator.predicates;
    quote! {
        pub use pumpkin_util::loot_table::*;
        #predicates
        #declarations
        static LOOT_TABLES_BY_KEY: [(&str, &LootTable); #count] = [#(#rows),*];
        #[must_use]
        pub fn get_loot_table(key: &str) -> Option<&'static LootTable> {
            LOOT_TABLES_BY_KEY.binary_search_by_key(&key, |&(key, _)| key).ok().map(|index| LOOT_TABLES_BY_KEY[index].1)
        }
        #[must_use]
        pub fn get_loot_predicate(key: &str) -> Option<&'static LootPredicateValue> {
            match key { #(#predicate_rows)* _ => None }
        }
        #[must_use]
        pub fn get_chest_loot_table(key: &str) -> Option<&'static LootTable> { get_loot_table(key) }
    }
}
