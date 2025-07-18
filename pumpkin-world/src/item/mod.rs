use pumpkin_data::Block;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component::DataComponent::{MaxStackSize, Tool};
use pumpkin_data::data_component_impl::IDSet;
use pumpkin_data::item::Item;
use pumpkin_data::recipes::RecipeResultStruct;
use pumpkin_data::tag::Taggable;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::GameMode;
use std::hash::Hash;

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
    pub item: &'static Item,
    pub patch: Vec<(u8, Option<DataComponent>)>,
}

impl Hash for ItemStack {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.item_count.hash(state);
        self.item.id.hash(state);
        self.patch.hash(state);
    }
}

/*
impl PartialEq for ItemStack {
    fn eq(&self, other: &Self) -> bool {
        self.item.id == other.item.id
    }
} */

impl ItemStack {
    pub fn new(item_count: u8, item: &'static Item) -> Self {
        Self {
            item_count,
            item,
            patch: Vec::new(),
        }
    }

    pub const EMPTY: &'static ItemStack = &ItemStack {
        item_count: 0,
        item: &Item::AIR,
        patch: Vec::new(),
    };

    pub fn get_max_stack_size(&self) -> u8 {
        for (id, component) in &self.patch {
            if id == &1 {
                if let Some(MaxStackSize(max)) = component {
                    return max.size;
                }
            }
        }
        for component in self.item.components {
            if let MaxStackSize(size) = component {
                return size.size;
            }
        }
        unreachable!();
    }

    pub fn get_item(&self) -> &Item {
        if self.is_empty() {
            &Item::AIR
        } else {
            self.item
        }
    }

    pub fn is_stackable(&self) -> bool {
        self.get_max_stack_size() > 1 // TODO: && (!this.isDamageable() || !this.isDamaged());
    }

    pub fn is_empty(&self) -> bool {
        self.item_count == 0 || self.item.id == Item::AIR.id
    }

    pub fn split(&mut self, amount: u8) -> Self {
        let min = amount.min(self.item_count);
        let stack = self.copy_with_count(min);
        self.decrement(min);
        stack
    }

    pub fn split_unless_creative(&mut self, gamemode: GameMode, amount: u8) -> Self {
        let min = amount.min(self.item_count);
        let stack = self.copy_with_count(min);
        if gamemode != GameMode::Creative {
            self.decrement(min);
        }
        stack
    }

    pub fn copy_with_count(&self, count: u8) -> Self {
        let mut stack = self.clone();
        stack.item_count = count;
        stack
    }

    pub fn set_count(&mut self, count: u8) {
        self.item_count = count;
    }

    pub fn decrement(&mut self, amount: u8) {
        self.item_count = self.item_count.saturating_sub(amount);
    }

    pub fn increment(&mut self, amount: u8) {
        self.item_count = self.item_count.saturating_add(amount);
    }

    pub fn are_items_and_components_equal(&self, other: &Self) -> bool {
        self.item == other.item //TODO: && self.item.components == other.item.components
    }

    pub fn are_equal(&self, other: &Self) -> bool {
        self.item_count == other.item_count && self.are_items_and_components_equal(other)
    }

    /// Determines the mining speed for a block based on tool rules.
    /// Direct matches return immediately, tagged blocks are checked separately.
    /// If no match is found, returns the tool's default mining speed or `1.0`.
    pub fn get_speed(&self, block: &'static Block) -> f32 {
        // No tool? Use default speed
        for component in self.item.components {
            if let Tool(tool) = component {
                for rule in tool.rules.iter() {
                    // Skip if speed is not set
                    let Some(speed) = rule.speed else {
                        continue;
                    };
                    match &rule.blocks {
                        IDSet::Tag(tag) => {
                            if block.is_tagged_with_by_tag(tag) {
                                return speed;
                            }
                        }
                        IDSet::Blocks(blocks) => {
                            if blocks.contains(&block) {
                                return speed;
                            }
                        }
                    }
                }
                return tool.default_mining_speed;
            }
        }
        1.0
    }

    /// Determines if a tool is valid for block drops based on tool rules.
    /// Direct matches return immediately, while tagged blocks are checked separately.
    pub fn is_correct_for_drops(&self, block: &'static Block) -> bool {
        // No tool? Use default speed
        for component in self.item.components {
            if let Tool(tool) = component {
                for rule in tool.rules.iter() {
                    // Skip if speed is not set
                    let Some(correct) = rule.correct_for_drops else {
                        continue;
                    };
                    match &rule.blocks {
                        IDSet::Tag(tag) => {
                            if block.is_tagged_with_by_tag(tag) {
                                return correct;
                            }
                        }
                        IDSet::Blocks(blocks) => {
                            if blocks.contains(&block) {
                                return correct;
                            }
                        }
                    }
                }
                return false;
            }
        }
        false
    }

    pub fn write_item_stack(&self, compound: &mut NbtCompound) {
        // Minecraft 1.21.4 uses "id" as string with namespaced ID (minecraft:diamond_sword)
        compound.put_string("id", format!("minecraft:{}", self.item.registry_key));
        compound.put_int("count", self.item_count as i32);

        // Create a tag compound for additional data
        let tag = NbtCompound::new();

        // TODO: Store custom data like enchantments, display name, etc. would go here

        // Store custom data like enchantments, display name, etc. would go here
        compound.put_component("components", tag);
    }

    pub fn read_item_stack(compound: &NbtCompound) -> Option<Self> {
        // Get ID, which is a string like "minecraft:diamond_sword"
        let full_id = compound.get_string("id")?;

        // Remove the "minecraft:" prefix if present
        let registry_key = full_id.strip_prefix("minecraft:").unwrap_or(full_id);

        // Try to get item by registry key
        let item = Item::from_registry_key(registry_key)?;

        let count = compound.get_int("count")? as u8;

        // Create the item stack
        let item_stack = Self::new(count, item);

        // Process any additional data in the components compound
        if let Some(_tag) = compound.get_compound("components") {
            // TODO: Process additional components like damage, enchantments, etc.
        }

        Some(item_stack)
    }
}

impl From<&RecipeResultStruct> for ItemStack {
    fn from(value: &RecipeResultStruct) -> Self {
        Self {
            item_count: value.count,
            item: Item::from_registry_key(value.id.strip_prefix("minecraft:").unwrap_or(value.id))
                .expect("Crafting recipe gives invalid item"),
            patch: Vec::new(),
        }
    }
}
