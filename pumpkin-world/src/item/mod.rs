use pumpkin_data::item::Item;

mod categories;
#[derive(serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
/// Item Rarity
pub enum Rarity {
    Common,
    UnCommon,
    Rare,
    Epic,
}

#[derive(Clone, Copy, Debug)]
pub struct ItemStack {
    pub item_count: u8,
    pub item: Item,
}

impl PartialEq for ItemStack {
    fn eq(&self, other: &Self) -> bool {
        self.item.id == other.item.id
    }
}

impl ItemStack {
    pub fn new(item_count: u8, item: Item) -> Self {
        Self { item_count, item }
    }

    pub fn get_speed(&self, block: &str) -> f32 {
        if let Some(tool) = self.item.components.tool {
            for rule in tool.rules {
                if rule.speed.is_none() || !rule.blocks.contains(&block) {
                    continue;
                }
                return rule.speed.unwrap();
            }
            return tool.default_mining_speed.unwrap_or(1.0);
        }
        1.0
    }

    pub fn is_correct_for_drops(&self, block: &str) -> bool {
        if let Some(tool) = self.item.components.tool {
            for rule in tool.rules {
                if rule.correct_for_drops.is_none() || !rule.blocks.contains(&block) {
                    continue;
                }
                return rule.correct_for_drops.unwrap();
            }
        }
        false
    }
}
