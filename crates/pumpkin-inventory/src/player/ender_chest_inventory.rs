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

use std::{any::Any, pin::Pin, sync::Arc};

use pumpkin_data::item_stack::ItemStack;
use pumpkin_world::{
    block::viewer::ViewerCountTracker,
    inventory::{Clearable, Inventory, InventoryFuture},
};
use tokio::sync::{Mutex, RwLock};

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
    pub async fn set_tracker(&self, tracker: Arc<ViewerCountTracker>) {
        self.tracker.lock().await.replace(tracker);
    }

    /// Checks if this inventory has a tracker set.
    pub async fn has_tracker(&self) -> bool {
        self.tracker.lock().await.is_some()
    }

    /// Checks if the given tracker is associated with this inventory.
    pub async fn is_tracker(&self, tracker: &Arc<ViewerCountTracker>) -> bool {
        if let Some(value) = self.tracker.lock().await.as_ref() {
            return Arc::ptr_eq(value, tracker);
        }
        false
    }
}

impl Inventory for EnderChestInventory {
    fn size(&self) -> usize {
        Self::INVENTORY_SIZE
    }

    fn is_empty(&self) -> InventoryFuture<'_, bool> {
        Box::pin(async move {
            let items = self.items.read().await;
            items.iter().all(ItemStack::is_empty)
        })
    }

    fn get_stack(&self, slot: usize) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            let items = self.items.read().await;
            items[slot].clone()
        })
    }

    fn remove_stack(&self, slot: usize) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            let mut items = self.items.write().await;
            std::mem::replace(&mut items[slot], ItemStack::EMPTY.clone())
        })
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            let mut items = self.items.write().await;
            if !items[slot].is_empty() && amount > 0 {
                items[slot].split(amount)
            } else {
                ItemStack::EMPTY.clone()
            }
        })
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            let mut items = self.items.write().await;
            items[slot] = stack;
        })
    }

    fn on_open(&self) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            if let Some(tracker) = self.tracker.lock().await.as_ref() {
                tracker.open_container();
            }
        })
    }

    fn on_close(&self) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            if let Some(tracker) = self.tracker.lock().await.as_ref() {
                tracker.close_container();
            }
        })
    }

    fn mark_dirty(&self) {}

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Clearable for EnderChestInventory {
    fn clear(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let mut items = self.items.write().await;
            items.fill_with(|| ItemStack::EMPTY.clone());
        })
    }
}
