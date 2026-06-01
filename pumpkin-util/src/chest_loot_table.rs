/// A single item entry inside a chest loot pool.
#[derive(Clone, Copy, Debug)]
pub struct ChestLootEntry {
    /// Registry name of the item (e.g. `"minecraft:diamond"`).
    pub item: &'static str,
    /// Relative probability weight; higher values are more likely.
    pub weight: i32,
    /// Minimum stack size (inclusive).
    pub min_count: i32,
    /// Maximum stack size (inclusive).
    pub max_count: i32,
}

/// One roll pool inside a chest loot table.
#[derive(Clone, Copy, Debug)]
pub struct ChestLootPool {
    /// Item entries eligible for selection each roll.
    pub entries: &'static [ChestLootEntry],
    /// Minimum number of roll attempts (inclusive).
    pub min_rolls: i32,
    /// Maximum number of roll attempts (inclusive).
    pub max_rolls: i32,
    /// Weight of the implicit "empty" (no item) outcome per roll.
    /// In vanilla this is modelled as a `minecraft:empty` entry with the given weight.
    pub empty_weight: i32,
}

/// A complete chest loot table consisting of one or more pools.
#[derive(Clone, Copy, Debug)]
pub struct ChestLootTable {
    /// All pools to roll when generating loot for this table.
    pub pools: &'static [ChestLootPool],
}
