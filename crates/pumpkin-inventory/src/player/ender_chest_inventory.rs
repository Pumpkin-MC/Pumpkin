//! Ender chest inventory implementation.
//!
//! Ender chests are player-specific storage that persist across dimensions.
//! Each player has their own ender chest contents that is accessible from
//! any ender chest block. The inventory syncs across all ender chests
//! for that player.
//!
//! # Viewer Tracking
//!
//! Ender chests track when players open and close them to properly
//! manage the viewer count for animation purposes.

use std::any::Any;
use std::sync::{Arc, Mutex, RwLock};

use pumpkin_data::item_stack::ItemStack;
use pumpkin_world::{
    block::viewer::ViewerCountTracker,
    inventory::{Clearable, Inventory},
};

/// A player's ender chest inventory.
///
/// Stores 27 slots (like a single chest) that are private to each player.
/// Contents persist across dimensions and are accessible from any
/// ender chest block.
pub struct EnderChestInventory {
    /// The 27 item slots in the ender chest.
    pub items: RwLock<[ItemStack; Self::INVENTORY_SIZE]>,
    /// Viewer count tracker for lid animation.
    ///
    /// Tracks how many players have the ender chest open to animate the lid.
    pub tracker: Mutex<Option<Arc<ViewerCountTracker>>>,
}

impl Default for EnderChestInventory {
    fn default() -> Self {
        Self::new()
    }
}

impl EnderChestInventory {
    /// The size of an ender chest inventory (27 slots).
    pub const INVENTORY_SIZE: usize = 27;

    /// Creates a new empty ender chest inventory.
    #[must_use]
    pub fn new() -> Self {
        Self {
            items: RwLock::new(std::array::from_fn(|_| ItemStack::EMPTY.clone())),
            tracker: Mutex::new(None),
        }
    }

    /// Sets the viewer count tracker for this inventory.
    ///
    /// Used to animate the ender chest lid based on viewers.
    pub fn set_tracker(&self, tracker: Arc<ViewerCountTracker>) {
        let old = self
            .tracker
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .replace(tracker);
        if let Some(old_tracker) = old {
            old_tracker.close_container();
        }
    }

    /// Checks if this inventory has a tracker set.
    pub fn has_tracker(&self) -> bool {
        self.tracker
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some()
    }

    /// Checks if the given tracker is associated with this inventory.
    pub fn is_tracker(&self, tracker: &Arc<ViewerCountTracker>) -> bool {
        if let Some(value) = self
            .tracker
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
        {
            return Arc::ptr_eq(value, tracker);
        }
        false
    }
}

impl Inventory for EnderChestInventory {
    fn size(&self) -> usize {
        Self::INVENTORY_SIZE
    }

    fn is_empty(&self) -> bool {
        let items = self
            .items
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        items.iter().all(ItemStack::is_empty)
    }

    fn get_stack(&self, slot: usize) -> ItemStack {
        let items = self
            .items
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        items
            .get(slot)
            .cloned()
            .unwrap_or_else(|| ItemStack::EMPTY.clone())
    }

    fn remove_stack(&self, slot: usize) -> ItemStack {
        let mut items = self
            .items
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if slot < Self::INVENTORY_SIZE {
            std::mem::replace(&mut items[slot], ItemStack::EMPTY.clone())
        } else {
            ItemStack::EMPTY.clone()
        }
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> ItemStack {
        let mut items = self
            .items
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if slot < Self::INVENTORY_SIZE && !items[slot].is_empty() && amount > 0 {
            items[slot].split(amount)
        } else {
            ItemStack::EMPTY.clone()
        }
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) {
        let mut items = self
            .items
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if slot < Self::INVENTORY_SIZE {
            items[slot] = stack;
        }
    }

    fn on_open(&self) {
        if let Some(tracker) = self
            .tracker
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
        {
            tracker.open_container();
        }
    }

    fn on_close(&self) {
        let tracker = self
            .tracker
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        if let Some(tracker) = tracker {
            tracker.close_container();
        }
    }

    fn mark_dirty(&self) {}

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Clearable for EnderChestInventory {
    fn clear(&self) {
        let mut items = self
            .items
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        items.fill_with(|| ItemStack::EMPTY.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::item::Item;

    #[test]
    fn new_inventory() {
        let ec = EnderChestInventory::new();
        assert_eq!(ec.size(), 27);
        assert!(ec.is_empty());
        assert!(!ec.has_tracker());
    }

    #[test]
    fn set_and_get_stack() {
        let ec = EnderChestInventory::new();
        let stack = ItemStack::new(1, &Item::DIRT);
        ec.set_stack(0, stack.clone());
        assert_eq!(ec.get_stack(0).item.id, Item::DIRT.id);
        assert_eq!(ec.get_stack(0).item_count, 1);
        assert!(!ec.is_empty());

        // Out of bounds shouldn't panic
        ec.set_stack(100, stack);
        assert!(ec.get_stack(100).is_empty());
    }

    #[test]
    fn remove_stack() {
        let ec = EnderChestInventory::new();
        let stack = ItemStack::new(5, &Item::DIAMOND);
        ec.set_stack(10, stack);

        let removed_specific = ec.remove_stack_specific(10, 2);
        assert_eq!(removed_specific.item_count, 2);
        assert_eq!(ec.get_stack(10).item_count, 3);

        let removed_all = ec.remove_stack(10);
        assert_eq!(removed_all.item_count, 3);
        assert!(ec.get_stack(10).is_empty());
        assert!(ec.is_empty());
    }

    #[test]
    fn clear_inventory() {
        let ec = EnderChestInventory::new();
        ec.set_stack(0, ItemStack::new(1, &Item::STONE));
        ec.set_stack(26, ItemStack::new(1, &Item::OAK_LOG));
        assert!(!ec.is_empty());

        ec.clear();
        assert!(ec.is_empty());
    }

    #[test]
    fn tracker_lifecycle() {
        let ec = EnderChestInventory::new();
        let tracker1 = Arc::new(ViewerCountTracker::new());
        let tracker2 = Arc::new(ViewerCountTracker::new());

        ec.set_tracker(tracker1.clone());
        assert!(ec.has_tracker());
        assert!(ec.is_tracker(&tracker1));
        assert!(!ec.is_tracker(&tracker2));

        ec.on_open();
        assert_eq!(tracker1.get_viewer_count(), 1);

        // Setting a new tracker while one is open should close the old tracker
        ec.set_tracker(tracker2.clone());
        assert_eq!(tracker1.get_viewer_count(), 0);
        assert!(ec.is_tracker(&tracker2));

        ec.on_open();
        assert_eq!(tracker2.get_viewer_count(), 1);

        ec.on_close();
        assert_eq!(tracker2.get_viewer_count(), 0);
        assert!(!ec.has_tracker());
    }
}
