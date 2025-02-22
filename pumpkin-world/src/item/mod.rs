use pumpkin_data::item::Item;
use pumpkin_data::tag::{RegistryKey, get_tag_values};

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

    /// Determines the mining speed for a block based on tool rules.
    /// Direct matches return immediately, tagged blocks are checked separately.
    /// If no match is found, returns the tool's default mining speed or `1.0`.
    pub fn get_speed(&self, block: &str) -> f32 {
        // No tool? Use default speed
        let Some(tool) = &self.item.components.tool else {
            return 1.0;
        };

        for rule in tool.rules {
            // Skip if speed is not set
            let Some(speed) = rule.speed else {
                continue;
            };

            for entry in rule.blocks {
                if entry.eq(&block) {
                    return speed;
                }

                if entry.starts_with('#') {
                    // Check if block is in the tag group
                    if let Some(blocks) =
                        get_tag_values(RegistryKey::Block, entry.strip_prefix('#').unwrap())
                    {
                        if blocks.iter().flatten().any(|s| s == block) {
                            return speed;
                        }
                    }
                }
            }
        }
        // Return default mining speed if no match is found
        tool.default_mining_speed.unwrap_or(1.0)
    }

    /// Determines if a tool is valid for block drops based on tool rules.
    /// Direct matches return immediately, while tagged blocks are checked separately.
    pub fn is_correct_for_drops(&self, block: &str) -> bool {
        // Return false if no tool component exists
        let Some(tool) = &self.item.components.tool else {
            return false;
        };

        for rule in tool.rules {
            // Skip rules without a drop condition
            let Some(correct_for_drops) = rule.correct_for_drops else {
                continue;
            };

            for entry in rule.blocks {
                if entry.eq(&block) {
                    return correct_for_drops;
                }

                if entry.starts_with('#') {
                    // Check if block exists within the tag group
                    if let Some(blocks) =
                        get_tag_values(RegistryKey::Block, entry.strip_prefix('#').unwrap())
                    {
                        if blocks.iter().flatten().any(|s| s == block) {
                            return correct_for_drops;
                        }
                    }
                }
            }
        }
        false
    }
}
