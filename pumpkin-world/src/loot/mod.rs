use condition::LootCondition;
use entry::LootPoolEntryTypes;
use serde::Deserialize;

use crate::item::ItemStack;

mod condition;
mod entry;

#[expect(dead_code)]
#[derive(Deserialize, Clone)]
pub struct LootTable {
    r#type: LootTableType,
    random_sequence: Option<String>,
    pools: Option<Vec<LootPool>>,
}

impl LootTable {
    pub fn get_loot(&self) -> Vec<ItemStack> {
        let mut items = vec![];
        if let Some(pools) = &self.pools {
            for pool in pools {
                items.extend_from_slice(&pool.get_loot());
            }
        }
        items
    }
}

#[derive(Deserialize, Clone)]
pub struct LootPool {
    entries: Vec<LootPoolEntry>,
    rolls: f32, // TODO
    bonus_rolls: f32,
}

impl LootPool {
    pub fn get_loot(&self) -> Vec<ItemStack> {
        let i = self.rolls.round() as i32 + self.bonus_rolls.floor() as i32; // TODO: mul by luck
        let mut items = vec![];
        for _ in 0..i {
            for entry in &self.entries {
                if let Some(conditions) = &entry.conditions {
                    if !conditions.iter().all(|condition| condition.test()) {
                        continue;
                    }
                }
                items.extend_from_slice(&entry.content.get_items());
            }
        }
        items
    }
}

#[derive(Deserialize, Clone)]
pub struct LootPoolEntry {
    #[serde(flatten)]
    content: LootPoolEntryTypes,
    conditions: Option<Vec<LootCondition>>,
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
