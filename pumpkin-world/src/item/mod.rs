use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component::DataComponent::Enchantments;
use pumpkin_data::data_component_impl::{
    BlocksAttacksImpl, ConsumableImpl, DataComponentImpl, EnchantmentsImpl, IDSet, MaxStackSizeImpl, ToolImpl, get,
    get_mut, read_data,
};
use pumpkin_data::item::Item;
use pumpkin_data::recipes::RecipeResultStruct;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, Enchantment};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::GameMode;
use std::borrow::Cow;
use std::cmp::{max, min};

mod categories;

#[derive(Clone, Debug)]
pub struct ItemStack {
    pub item_count: u8,
    pub item: &'static Item,
    pub patch: Vec<(DataComponent, Option<Box<dyn DataComponentImpl>>)>,
}

// impl Hash for ItemStack {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         self.item_count.hash(state);
//         self.item.id.hash(state);
//         self.patch.hash(state);
//     }
// }

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

    pub fn new_with_component(
        item_count: u8,
        item: &'static Item,
        component: Vec<(DataComponent, Option<Box<dyn DataComponentImpl>>)>,
    ) -> Self {
        Self {
            item_count,
            item,
            patch: component,
        }
    }

    pub fn get_data_component<T: DataComponentImpl + 'static>(&self) -> Option<&T> {
        let to_get_id = &T::get_enum();
        for (id, component) in &self.patch {
            if id == to_get_id {
                return if let Some(component) = component {
                    Some(get::<T>(component.as_ref()))
                } else {
                    None
                };
            }
        }
        for (id, component) in self.item.components {
            if id == to_get_id {
                return Some(get::<T>(*component));
            }
        }
        None
    }
    pub fn get_data_component_mut<T: DataComponentImpl + 'static>(&mut self) -> Option<&mut T> {
        let to_get_id = &T::get_enum();
        for (id, component) in self.patch.iter_mut() {
            if id == to_get_id {
                return if let Some(component) = component {
                    Some(get_mut::<T>(component.as_mut()))
                } else {
                    None
                };
            }
        }
        None
    }

    pub const EMPTY: &'static ItemStack = &ItemStack {
        item_count: 0,
        item: &Item::AIR,
        patch: Vec::new(),
    };

    pub fn get_max_stack_size(&self) -> u8 {
        if let Some(value) = self.get_data_component::<MaxStackSizeImpl>() {
            value.size
        } else {
            1
        }
    }

    pub fn get_max_use_time(&self) -> i32 {
        if let Some(value) = self.get_data_component::<ConsumableImpl>() {
            return value.consume_ticks();
        }
        if self.get_data_component::<BlocksAttacksImpl>().is_some() {
            return 72000;
        }
        0
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

    pub fn decrement_unless_creative(&mut self, gamemode: GameMode, amount: u8) {
        if gamemode != GameMode::Creative {
            self.item_count = self.item_count.saturating_sub(amount);
        }
    }

    pub fn decrement(&mut self, amount: u8) {
        self.item_count = self.item_count.saturating_sub(amount);
    }

    pub fn increment(&mut self, amount: u8) {
        self.item_count = self.item_count.saturating_add(amount);
    }

    pub fn enchant(&mut self, enchantment: &'static Enchantment, level: i32) {
        // TODO itemstack may not send update packet to client
        if level <= 0 {
            return;
        }
        let level = min(level, 255);
        if let Some(data) = self.get_data_component_mut::<EnchantmentsImpl>() {
            for (enc, old_level) in data.enchantment.to_mut() {
                if *enc == enchantment {
                    *old_level = max(*old_level, level);
                    return;
                }
            }
            data.enchantment.to_mut().push((enchantment, level));
        } else {
            self.patch.push((
                Enchantments,
                Some(
                    EnchantmentsImpl {
                        enchantment: Cow::Owned(vec![(enchantment, level)]),
                    }
                    .to_dyn(),
                ),
            ));
        }
    }
    pub fn are_items_and_components_equal(&self, other: &Self) -> bool {
        if self.item != other.item || self.patch.len() != other.patch.len() {
            return false;
        }
        for (id, data) in &self.patch {
            let mut not_find = true;
            'out: for (other_id, other_data) in &other.patch {
                if id == other_id {
                    if let (Some(data), Some(other_data)) = (data, other_data) {
                        if data.equal(other_data.as_ref()) {
                            return false;
                        } else {
                            not_find = false;
                            break 'out;
                        }
                    } else if data.is_none() && other_data.is_none() {
                        not_find = false;
                        break 'out;
                    } else {
                        return false;
                    }
                }
            }
            if not_find {
                return false;
            }
        }
        true
    }

    pub fn are_equal(&self, other: &Self) -> bool {
        self.item_count == other.item_count && self.are_items_and_components_equal(other)
    }

    /// Determines the mining speed for a block based on tool rules.
    /// Direct matches return immediately, tagged blocks are checked separately.
    /// If no match is found, returns the tool's default mining speed or `1.0`.
    pub fn get_speed(&self, block: &'static Block) -> f32 {
        // No tool? Use default speed
        if let Some(tool) = self.get_data_component::<ToolImpl>() {
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
            tool.default_mining_speed
        } else {
            1.0
        }
    }

    /// Determines if a tool is valid for block drops based on tool rules.
    /// Direct matches return immediately, while tagged blocks are checked separately.
    pub fn is_correct_for_drops(&self, block: &'static Block) -> bool {
        if let Some(tool) = self.get_data_component::<ToolImpl>() {
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
            false
        } else {
            false
        }
    }

    pub fn write_item_stack(&self, compound: &mut NbtCompound) {
        // Minecraft 1.21.4 uses "id" as string with namespaced ID (minecraft:diamond_sword)
        compound.put_string("id", format!("minecraft:{}", self.item.registry_key));
        compound.put_int("count", self.item_count as i32);

        // Create a tag compound for additional data
        let mut tag = NbtCompound::new();

        for (id, data) in &self.patch {
            if let Some(data) = data {
                tag.put(id.to_name(), data.write_data());
            } else {
                let name = '!'.to_string() + id.to_name();
                tag.put(name.as_str(), NbtCompound::new());
            }
        }

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
        let mut item_stack = Self::new(count, item);

        // Process any additional data in the components compound
        if let Some(tag) = compound.get_compound("components") {
            for (name, data) in &tag.child_tags {
                if let Some(name) = name.strip_prefix("!") {
                    item_stack
                        .patch
                        .push((DataComponent::try_from_name(name)?, None));
                } else {
                    let id = DataComponent::try_from_name(name)?;
                    item_stack.patch.push((id, Some(read_data(id, data)?)));
                }
            }
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
