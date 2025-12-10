use std::{any::Any, array::from_fn, pin::Pin, sync::Arc};

use pumpkin_world::{
    block::viewer::ViewerCountTracker,
    inventory::{Clearable, Inventory, InventoryFuture, split_stack},
    item::ItemStack,
};
use tokio::sync::Mutex;

pub struct EnderChestInventory {
    pub items: [Arc<Mutex<ItemStack>>; Self::INVENTORY_SIZE],
    pub tracker: Mutex<Option<Arc<ViewerCountTracker>>>,
}

impl Default for EnderChestInventory {
    fn default() -> Self {
        Self::new()
    }
}

impl EnderChestInventory {
    pub const INVENTORY_SIZE: usize = 27;

    pub fn new() -> Self {
        Self {
            items: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY.clone()))),
            tracker: Mutex::new(None),
        }
    }

    pub async fn set_tracker(&self, tracker: Arc<ViewerCountTracker>) {
        self.tracker.lock().await.replace(tracker);
    }

    pub async fn has_tracker(&self) -> bool {
        self.tracker.lock().await.is_some()
    }

    pub async fn is_tracker(&self, tracker: &Arc<ViewerCountTracker>) -> bool {
        if let Some(value) = self.tracker.lock().await.as_ref() {
            return Arc::ptr_eq(value, tracker);
        }
        false
    }
}

impl Inventory for EnderChestInventory {
    fn size(&self) -> usize {
        self.items.len()
    }

    fn is_empty(&self) -> InventoryFuture<'_, bool> {
        Box::pin(async move {
            for slot in self.items.iter() {
                if !slot.lock().await.is_empty() {
                    return false;
                }
            }

            true
        })
    }

    fn get_stack(&self, slot: usize) -> InventoryFuture<'_, Arc<Mutex<ItemStack>>> {
        Box::pin(async move { self.items[slot].clone() })
    }

    fn remove_stack(&self, slot: usize) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            let mut removed = ItemStack::EMPTY.clone();
            let mut guard = self.items[slot].lock().await;
            std::mem::swap(&mut removed, &mut *guard);
            removed
        })
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move { split_stack(&self.items, slot, amount).await })
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            *self.items[slot].lock().await = stack;
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
            for item in self.items.iter() {
                *item.lock().await = ItemStack::EMPTY.clone();
            }
        })
    }
}
