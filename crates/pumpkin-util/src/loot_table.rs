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
    AllOf(&'static [Self]),
}

/// Bonus count formulas when tools have fortune or looting enchantments.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LootBonusFormula {
    OreDrops,
    UniformBonusCount(i32),
    BinomialWithBonusCount { extra: i32, probability: f32 },
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
    /// Condition required for this entry to be eligible.
    pub condition: LootCondition,
    /// Bonus formula to apply with fortune / looting (if any).
    pub bonus_formula: Option<LootBonusFormula>,
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
