use serde::Deserialize;

use crate::{item::ItemStack, loot::LootPoolEntry};

#[derive(Deserialize, Clone)]
pub struct AlternativeEntry {
    children: Vec<LootPoolEntry>,
}

impl AlternativeEntry {
    pub fn get_items(&self) -> Vec<ItemStack> {
        let mut items = vec![];
        for child in &self.children {
            if let Some(conditions) = &child.conditions {
                let mut conditions_met = true;
                for condition in conditions {
                    if !condition.test() {
                        conditions_met = false;
                        break;
                    }
                }
                if !conditions_met {
                    continue;
                }
            }
            items.extend_from_slice(&child.content.get_items());
        }
        items
    }
}
