use std::borrow::Cow;

use serde_json::Value;

/// Conditions required for an entry or pool to be eligible for loot generation.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum LootCondition {
    #[default]
    None,
    SilkTouch,
    NoSilkTouch,
    Shears,
    SilkTouchOrShears,
    NoSilkTouchOrShears,
    SurvivesExplosion,
    KilledByPlayer,
    RandomChance {
        chance: f32,
    },
    RandomChanceWithEnchantedBonus {
        unenchanted_chance: f32,
        enchanted_chance_base: f32,
        enchanted_chance_per_level_above_first: f32,
    },
    TableBonus {
        chances: &'static [f32],
    },
    AllOf(&'static [Self]),
}

/// Bonus count formulas when tools have fortune or looting enchantments.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LootBonusFormula {
    OreDrops,
    UniformBonusCount(i32),
    BinomialWithBonusCount { extra: i32, probability: f32 },
}

#[derive(Clone, Debug)]
pub struct LootStateCount {
    pub block: Cow<'static, str>,
    pub properties: Cow<'static, [(Cow<'static, str>, Cow<'static, str>)]>,
    pub add: bool,
    pub min_count: i32,
    pub max_count: i32,
    pub binomial: Option<f32>,
}

impl LootStateCount {
    /// Builds a modifier from a `set_count`. `None` for a plain count or a non-`match_block` condition.
    #[must_use]
    pub fn parse(
        condition: Option<&Value>,
        add: bool,
        min_count: i32,
        max_count: i32,
        binomial: Option<f32>,
    ) -> Option<Self> {
        let (block, properties) = match condition {
            None if add => (String::new(), Vec::new()),
            None => return None,
            Some(condition) => {
                let kind = condition.get("type")?.as_str()?;
                if kind.strip_prefix("minecraft:").unwrap_or(kind) != "match_block" {
                    return None;
                }
                let block = condition.get("blocks")?.as_str()?.to_string();
                let state = condition.get("state").and_then(Value::as_object);
                let properties = state.into_iter().flatten().filter_map(|(name, value)| {
                    Some((
                        Cow::Owned(name.clone()),
                        Cow::Owned(value.as_str()?.to_string()),
                    ))
                });
                (block, properties.collect())
            }
        };
        Some(Self {
            block: Cow::Owned(block),
            properties: Cow::Owned(properties),
            add,
            min_count,
            max_count,
            binomial,
        })
    }
}

/// A single item entry inside a loot pool.
#[derive(Clone, Copy, Debug)]
pub struct LootEntry {
    /// Registry name of the item (e.g. `"minecraft:diamond"`).
    pub item: &'static str,
    /// Relative probability weight; higher values are more likely.
    pub weight: i32,
    /// Minimum stack size (inclusive).
    pub min_count: i32,
    /// Maximum stack size (inclusive).
    pub max_count: i32,
    /// Conditional counts applied when the broken block state matches.
    pub state_counts: &'static [LootStateCount],
    /// Condition required for this entry to be eligible.
    pub condition: LootCondition,
    /// Bonus formula to apply with fortune / looting (if any).
    pub bonus_formula: Option<LootBonusFormula>,
    /// Block state the bonus formula is gated on. Only `block` and `properties` are used.
    pub bonus_condition: Option<&'static LootStateCount>,
}

/// One roll pool inside a loot table.
#[derive(Clone, Copy, Debug)]
pub struct LootPool {
    /// Item entries eligible for selection each roll.
    pub entries: &'static [LootEntry],
    /// Minimum number of roll attempts (inclusive).
    pub min_rolls: i32,
    /// Maximum number of roll attempts (inclusive).
    pub max_rolls: i32,
    /// Weight of the implicit "empty" (no item) outcome per roll.
    /// In vanilla this is modelled as a `minecraft:empty` entry with the given weight.
    pub empty_weight: i32,
    /// Condition required for this entire pool to run.
    pub condition: LootCondition,
}

/// A complete loot table consisting of one or more pools.
#[derive(Clone, Copy, Debug)]
pub struct LootTable {
    /// All pools to roll when generating loot for this table.
    pub pools: &'static [LootPool],
}

pub type ChestLootEntry = LootEntry;
pub type ChestLootPool = LootPool;
pub type ChestLootTable = LootTable;

/// Conditions required for an entry or pool to be eligible for dynamic loot generation.
#[derive(Clone, Debug, PartialEq, Default)]
pub enum DynamicLootCondition {
    #[default]
    None,
    SilkTouch,
    NoSilkTouch,
    Shears,
    SilkTouchOrShears,
    NoSilkTouchOrShears,
    SurvivesExplosion,
    KilledByPlayer,
    RandomChance {
        chance: f32,
    },
    RandomChanceWithEnchantedBonus {
        unenchanted_chance: f32,
        enchanted_chance_base: f32,
        enchanted_chance_per_level_above_first: f32,
    },
    TableBonus {
        chances: Box<[f32]>,
    },
    AllOf(Vec<Self>),
    AnyOf(Vec<Self>),
    Inverted(Box<Self>),
    EntityOnFire,
    WeatherCheck {
        raining: Option<bool>,
        thundering: Option<bool>,
    },
}

impl From<LootCondition> for DynamicLootCondition {
    fn from(cond: LootCondition) -> Self {
        match cond {
            LootCondition::None => Self::None,
            LootCondition::SilkTouch => Self::SilkTouch,
            LootCondition::NoSilkTouch => Self::NoSilkTouch,
            LootCondition::Shears => Self::Shears,
            LootCondition::SilkTouchOrShears => Self::SilkTouchOrShears,
            LootCondition::NoSilkTouchOrShears => Self::NoSilkTouchOrShears,
            LootCondition::SurvivesExplosion => Self::SurvivesExplosion,
            LootCondition::KilledByPlayer => Self::KilledByPlayer,
            LootCondition::RandomChance { chance } => Self::RandomChance { chance },
            LootCondition::RandomChanceWithEnchantedBonus {
                unenchanted_chance,
                enchanted_chance_base,
                enchanted_chance_per_level_above_first,
            } => Self::RandomChanceWithEnchantedBonus {
                unenchanted_chance,
                enchanted_chance_base,
                enchanted_chance_per_level_above_first,
            },
            LootCondition::TableBonus { chances } => Self::TableBonus {
                chances: chances.to_vec().into_boxed_slice(),
            },
            LootCondition::AllOf(conditions) => {
                Self::AllOf(conditions.iter().copied().map(Self::from).collect())
            }
        }
    }
}

/// A single item entry inside a dynamic loot pool.
#[derive(Clone, Debug)]
pub struct DynamicLootEntry {
    /// Registry name of the item (e.g. `"minecraft:diamond"`).
    pub item: String,
    /// Relative probability weight; higher values are more likely.
    pub weight: i32,
    /// Minimum stack size (inclusive).
    pub min_count: i32,
    /// Maximum stack size (inclusive).
    pub max_count: i32,
    /// Conditional `set_count` modifiers, applied in order when the block state matches.
    pub state_counts: Vec<LootStateCount>,
    /// Condition required for this entry to be eligible.
    pub condition: DynamicLootCondition,
    /// Bonus formula to apply with fortune / looting (if any).
    pub bonus_formula: Option<LootBonusFormula>,
    /// Block state the bonus formula is gated on. Only `block` and `properties` are used.
    pub bonus_condition: Option<LootStateCount>,
}

impl From<LootEntry> for DynamicLootEntry {
    fn from(e: LootEntry) -> Self {
        Self {
            item: e.item.to_string(),
            weight: e.weight,
            min_count: e.min_count,
            max_count: e.max_count,
            state_counts: e.state_counts.to_vec(),
            condition: DynamicLootCondition::from(e.condition),
            bonus_formula: e.bonus_formula,
            bonus_condition: e.bonus_condition.cloned(),
        }
    }
}

/// One roll pool inside a dynamic loot table.
#[derive(Clone, Debug)]
pub struct DynamicLootPool {
    /// Item entries eligible for selection each roll.
    pub entries: Vec<DynamicLootEntry>,
    /// Minimum number of roll attempts (inclusive).
    pub min_rolls: i32,
    /// Maximum number of roll attempts (inclusive).
    pub max_rolls: i32,
    /// Weight of the implicit "empty" (no item) outcome per roll.
    pub empty_weight: i32,
    /// Condition required for this entire pool to run.
    pub condition: DynamicLootCondition,
}

impl From<LootPool> for DynamicLootPool {
    fn from(p: LootPool) -> Self {
        Self {
            entries: p
                .entries
                .iter()
                .copied()
                .map(DynamicLootEntry::from)
                .collect(),
            min_rolls: p.min_rolls,
            max_rolls: p.max_rolls,
            empty_weight: p.empty_weight,
            condition: DynamicLootCondition::from(p.condition),
        }
    }
}

/// A complete dynamic loot table consisting of one or more pools.
#[derive(Clone, Debug, Default)]
pub struct DynamicLootTable {
    /// All pools to roll when generating loot for this table.
    pub pools: Vec<DynamicLootPool>,
}

impl From<&LootTable> for DynamicLootTable {
    fn from(t: &LootTable) -> Self {
        Self {
            pools: t.pools.iter().copied().map(DynamicLootPool::from).collect(),
        }
    }
}
