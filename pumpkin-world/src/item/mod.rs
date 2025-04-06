use pumpkin_data::item::Item;
use pumpkin_data::tag::{RegistryKey, get_tag_values};
use pumpkin_nbt::compound::NbtCompound;

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

#[derive(Clone, Debug)]
pub struct ItemStack {
    pub item_count: u8,
    // TODO: Should this be a ref? all of our items are const
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

    pub fn damage_item(&mut self) {
        let components = &mut self.item.components;

        if let (Some(_damage), Some(_max_damage)) = (components.damage, components.max_damage) {
            if _max_damage == 0 {
                return;
            }

            // TODO: we probably have to consider the unbreakable enchantment here
            if _damage >= _max_damage {
                return;
            }

            components.damage = Some(_damage + 1);
        }
    }

    pub fn is_broken(&self) -> bool {
        let components = self.item.components;

        // TODO: we probably have to consider the unbreakable enchantment here
        if let (Some(_damage), Some(_max_damage)) = (components.damage, components.max_damage) {
            return _max_damage > 0 && _damage >= _max_damage;
        }

        false
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
                        if blocks.iter().any(|s| *s == block) {
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
                        if blocks.iter().any(|s| *s == block) {
                            return correct_for_drops;
                        }
                    }
                }
            }
        }
        false
    }

    pub fn write_item_stack(&self, compound: &mut NbtCompound) {
        // Minecraft 1.21.4 uses "id" as string with namespaced ID (minecraft:diamond_sword)
        compound.put_string("id", format!("minecraft:{}", self.item.registry_key));
        compound.put_int("count", self.item_count as i32);

        // Create a tag compound for additional data
        let mut tag = NbtCompound::new();

        // TODO: Store custom data like enchantments, display name, etc.
        if let Some(damage) = self.item.components.damage {
            tag.put_int("damage", damage as i32);
        }

        if let Some(max_damage) = self.item.components.max_damage {
            tag.put_int("max_damage", max_damage as i32);
        }

        compound.put_component("components", tag);
    }

    pub fn read_item_stack(compound: &NbtCompound) -> Option<Self> {
        // Get ID, which is a string like "minecraft:diamond_sword"
        let full_id = compound.get_string("id")?;

        // Remove the "minecraft:" prefix if present
        let registry_key = full_id.strip_prefix("minecraft:").unwrap_or(full_id);

        // Try to get item by registry key
        let mut item = Item::from_registry_key(registry_key)?.clone();

        // TODO: Process additional components like damage, enchantments, etc.
        if let Some(_tag) = compound.get_compound("components") {
            if let Some(_damage) = _tag.get_int("damage") {
                item.components.damage = Some(_damage as u16);
            }

            if let Some(_max_damage) = _tag.get_int("max_damage") {
                item.components.max_damage = Some(_max_damage as u16);
            }
        }

        let count = compound.get_int("count")? as u8;

        Some(Self::new(count, item))
    }
}
