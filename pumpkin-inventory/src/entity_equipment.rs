//! Entity equipment management.
//!
//! This module handles the storage and management of entity equipment slots,
//! such as armor (head, chest, legs, feet) and off-hand items.
//!
//! Equipment is stored separately from the main inventory and is visible on
//! the entity model (armor is rendered on the player, held items are visible
//! in hands).

use std::{collections::HashMap, sync::Arc};

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::item_stack::ItemStack;
use tokio::sync::Mutex;

/// Equipment storage for an entity.
///
/// Stores items equipped in armor slots (head, chest, legs, feet) and
/// the off-hand slot. Equipment is separate from the main inventory
/// and affects entity appearance and stats.
///
/// See also: [`EquipmentSlot`](EquipmentSlot)
// EntityEquipment.java
#[derive(Clone)]
pub struct EntityEquipment {
    /// Map of equipment slots to their equipped items.
    ///
    /// Keys are equipment slot types (head, chest, legs, feet, off-hand).
    /// Values are mutex-protected item stacks for thread-safe access.
    pub equipment: HashMap<EquipmentSlot, Arc<Mutex<ItemStack>>>,
}

impl Default for EntityEquipment {
    fn default() -> Self {
        Self::new()
    }
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
    pub async fn put(&mut self, slot: &EquipmentSlot, stack: ItemStack) -> ItemStack {
        self.equipment
            .insert(slot.clone(), Arc::new(Mutex::new(stack)))
            .unwrap_or(Arc::new(Mutex::new(ItemStack::EMPTY.clone())))
            .lock()
            .await
            .clone()
    }

    /// Gets or inserts an empty stack for a slot.
    ///
    /// If the slot doesn't exist, creates it with an empty stack.
    ///
    /// # Returns
    /// A mutex-protected item stack for this slot.
    #[must_use]
    pub fn get_or_insert(&mut self, slot: &EquipmentSlot) -> Arc<Mutex<ItemStack>> {
        self.equipment
            .entry(slot.clone())
            .or_insert_with(|| Arc::new(Mutex::new(ItemStack::EMPTY.clone())))
            .clone()
    }

    /// Gets the item in a slot.
    ///
    /// # Returns
    /// The equipped item, or an empty stack if nothing is equipped.
    #[must_use]
    pub fn get(&self, slot: &EquipmentSlot) -> Arc<Mutex<ItemStack>> {
        self.equipment
            .get(slot)
            .cloned()
            .unwrap_or(Arc::new(Mutex::new(ItemStack::EMPTY.clone())))
    }

    /// Checks if all equipment slots are empty.
    pub async fn is_empty(&self) -> bool {
        for stack in self.equipment.values() {
            if !stack.lock().await.is_empty() {
                return false;
            }
        }

        true
    }

    /// Clears all equipped items.
    pub fn clear(&mut self) {
        self.equipment.clear();
    }

    // TODO: tick - Equipment updates, durability damage, etc.
}
