use serde::Deserialize;

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
}

#[expect(dead_code)]
#[derive(Deserialize, Clone)]
pub struct LootPoolEntry {
    // TODO
    r#type: Option<String>,
    // TODO
    name: Option<String>,
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
