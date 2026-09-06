//! Double inventory implementation.
//!
//! This module provides a composite inventory that combines two inventories
//! into one. This is used for large containers like double chests, which
//! consist of two single chest inventories viewed as a single 54-slot inventory.
//!
//! The first inventory's slots come first, followed by the second inventory's
//! slots. Operations are delegated to the appropriate underlying inventory
//! based on the slot index.

use std::{any::Any, sync::Arc};

use pumpkin_data::item_stack::ItemStack;
use pumpkin_world::inventory::{Clearable, Inventory};

/// A composite inventory combining two inventories.
///
/// Used for double chests and other large containers that span
/// multiple block entities. The combined inventory size is the sum
/// of both inventories' sizes.
pub struct DoubleInventory {
    /// The first inventory (lower slot indices, 0 to first.size()-1).
    first: Arc<dyn Inventory>,
    /// The second inventory (higher slot indices, `first.size()` to total-1).
    second: Arc<dyn Inventory>,
}

impl DoubleInventory {
    /// Creates a new double inventory.
    ///
    /// # Arguments
    /// - `first` - The first inventory (lower slot indices)
    /// - `second` - The second inventory (higher slot indices)
    ///
    /// # Returns
    /// A shared reference to the new double inventory.
    pub fn new(first: Arc<dyn Inventory>, second: Arc<dyn Inventory>) -> Arc<Self> {
        Arc::new(Self { first, second })
    }
}

impl Inventory for DoubleInventory {
    fn size(&self) -> usize {
        self.first.size() + self.second.size()
    }

    fn is_empty(&self) -> bool {
        self.first.is_empty() && self.second.is_empty()
    }

    fn get_stack(&self, slot: usize) -> ItemStack {
        if slot >= self.first.size() {
            self.second.get_stack(slot - self.first.size())
        } else {
            self.first.get_stack(slot)
        }
    }

    fn remove_stack(&self, slot: usize) -> ItemStack {
        if slot >= self.first.size() {
            self.second.remove_stack(slot - self.first.size())
        } else {
            self.first.remove_stack(slot)
        }
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> ItemStack {
        if slot >= self.first.size() {
            self.second
                .remove_stack_specific(slot - self.first.size(), amount)
        } else {
            self.first.remove_stack_specific(slot, amount)
        }
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) {
        if slot >= self.first.size() {
            self.second.set_stack(slot - self.first.size(), stack);
        } else {
            self.first.set_stack(slot, stack);
        }
    }

    fn on_open(&self) {
        self.first.on_open();
        self.second.on_open();
    }

    fn on_close(&self) {
        self.first.on_close();
        self.second.on_close();
    }

    fn get_max_count_per_stack(&self) -> u8 {
        self.first.get_max_count_per_stack()
    }

    fn mark_dirty(&self) {
        self.first.mark_dirty();
        self.second.mark_dirty();
    }

    fn is_valid_slot_for(&self, slot: usize, stack: &ItemStack) -> bool {
        if slot >= self.first.size() {
            self.second
                .is_valid_slot_for(slot - self.first.size(), stack)
        } else {
            self.first.is_valid_slot_for(slot, stack)
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Clearable for DoubleInventory {
    fn clear(&self) {
        self.first.clear();
        self.second.clear();
    }
}
