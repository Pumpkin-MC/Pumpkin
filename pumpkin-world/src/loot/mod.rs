use entry::LootPoolEntryTypes;
use serde::Deserialize;

mod entry;

#[expect(dead_code)]
#[derive(Deserialize, Clone)]
pub struct LootTable {
    r#type: LootTableType,
    pools: Option<Vec<LootPool>>,
}

#[expect(dead_code)]
#[derive(Deserialize, Clone)]
pub struct LootPool {
    entries: Vec<LootPoolEntry>,
    rolls: f32, // TODO
    bonus_rolls: f32,
}

#[expect(dead_code)]
#[derive(Deserialize, Clone)]
pub struct LootPoolEntry {
    #[serde(flatten)]
    content: LootPoolEntryTypes,
}

#[derive(Deserialize, Clone)]
#[serde(rename = "snake_case")]
pub enum LootTableType {
    #[serde(rename = "minecraft:empty")]
    /// Nothing will be dropped
    Empty,
    #[serde(rename = "minecraft:block")]
    /// A Block will be dropped
    Block,
    #[serde(rename = "minecraft:chest")]
    /// A Item will be dropped
    Chest,
}
