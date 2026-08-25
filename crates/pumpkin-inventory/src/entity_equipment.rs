//! Entity equipment management.
//!
//! This module handles the storage and management of entity equipment slots,
//! such as armor (head, chest, legs, feet) and off-hand items.
//!
//! Equipment is stored separately from the main inventory and is visible on
//! the entity model (armor is rendered on the player, held items are visible
//! in hands).

use std::collections::HashMap;

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::item_stack::ItemStack;

/// Equipment storage for an entity.
///
/// Stores items equipped in armor slots (head, chest, legs, feet) and
/// the off-hand slot. Equipment is separate from the main inventory
/// and affects entity appearance and stats.
///
/// See also: [`EquipmentSlot`](EquipmentSlot)
// EntityEquipment.java
#[derive(Clone, Default)]
pub struct EntityEquipment {
    /// Map of equipment slots to their equipped items.
    ///
    /// Keys are equipment slot types (head, chest, legs, feet, off-hand).
    pub equipment: HashMap<EquipmentSlot, ItemStack>,
}

impl EntityEquipment {
    /// Creates a new empty equipment storage.
    #[must_use]
    pub fn new() -> Self {
        Self {
            equipment: HashMap::new(),
        }
    }

    /// Equips an item in a slot, returning the previous item.
    ///
    /// # Arguments
    /// - `slot` - The equipment slot
    /// - `stack` - The item to equip
    ///
    /// # Returns
    /// The previously equipped item, or an empty stack if the slot was empty.
    pub fn put(&mut self, slot: &EquipmentSlot, stack: ItemStack) -> ItemStack {
        self.equipment
            .insert(slot.clone(), stack)
            .unwrap_or_else(|| ItemStack::EMPTY.clone())
    }

    /// Gets the item in a slot.
    ///
    /// # Returns
    /// The equipped item, or an empty stack if nothing is equipped.
    #[must_use]
    pub fn get(&self, slot: &EquipmentSlot) -> ItemStack {
        self.equipment
            .get(slot)
            .cloned()
            .unwrap_or_else(|| ItemStack::EMPTY.clone())
    }

    /// Checks if all equipment slots are empty.
    pub fn is_empty(&self) -> bool {
        self.equipment.values().all(ItemStack::is_empty)
    }

    /// Clears all equipped items.
    pub fn clear(&mut self) {
        self.equipment.clear();
    }

    // TODO: tick - Equipment updates, durability damage, etc.
}
