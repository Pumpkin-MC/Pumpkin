use std::{any::Any, array::from_fn, sync::Arc};

use async_trait::async_trait;
use pumpkin_world::{
    block::viewer::ViewerCountTracker,
    inventory::{Clearable, Inventory, split_stack},
    item::ItemStack,
};
use tokio::sync::Mutex;

#[derive(Debug)]
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

#[async_trait]
impl Inventory for EnderChestInventory {
    fn size(&self) -> usize {
        self.items.len()
    }

    async fn is_empty(&self) -> bool {
        for slot in self.items.iter() {
            if !slot.lock().await.is_empty() {
                return false;
            }
        }

        true
    }

    async fn get_stack(&self, slot: usize) -> Arc<Mutex<ItemStack>> {
        self.items[slot].clone()
    }

    async fn remove_stack(&self, slot: usize) -> ItemStack {
        let mut removed = ItemStack::EMPTY.clone();
        let mut guard = self.items[slot].lock().await;
        std::mem::swap(&mut removed, &mut *guard);
        removed
    }

    async fn remove_stack_specific(&self, slot: usize, amount: u8) -> ItemStack {
        split_stack(&self.items, slot, amount).await
    }

    async fn set_stack(&self, slot: usize, stack: ItemStack) {
        *self.items[slot].lock().await = stack;
    }

    async fn on_open(&self) {
        if let Some(tracker) = self.tracker.lock().await.as_ref() {
            tracker.open_container();
        }
    }

    async fn on_close(&self) {
        if let Some(tracker) = self.tracker.lock().await.as_ref() {
            tracker.close_container();
        }
    }

    fn mark_dirty(&self) {}

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
impl Clearable for EnderChestInventory {
    async fn clear(&self) {
        for slot in self.items.iter() {
            *slot.lock().await = ItemStack::EMPTY.clone();
        }
    }
}
