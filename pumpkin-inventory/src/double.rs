use std::{any::Any, sync::Arc};

use async_trait::async_trait;
use pumpkin_world::{
    inventory::{Clearable, Inventory},
    item::ItemStack,
};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct DoubleInventory {
    first: Arc<dyn Inventory>,
    second: Arc<dyn Inventory>,
}

impl DoubleInventory {
    pub fn new(first: Arc<dyn Inventory>, second: Arc<dyn Inventory>) -> Arc<Self> {
        Arc::new(Self { first, second })
    }
}

#[async_trait]
impl Inventory for DoubleInventory {
    fn size(&self) -> usize {
        self.first.size() + self.second.size()
    }

    async fn is_empty(&self) -> bool {
        self.first.is_empty().await && self.second.is_empty().await
    }

    async fn get_stack(&self, slot: usize) -> Arc<Mutex<ItemStack>> {
        if slot >= self.first.size() {
            self.second.get_stack(slot - self.first.size()).await
        } else {
            self.first.get_stack(slot).await
        }
    }

    async fn remove_stack(&self, slot: usize) -> ItemStack {
        if slot >= self.first.size() {
            self.second.remove_stack(slot - self.first.size()).await
        } else {
            self.first.remove_stack(slot).await
        }
    }

    async fn remove_stack_specific(&self, slot: usize, amount: u8) -> ItemStack {
        if slot >= self.first.size() {
            self.second
                .remove_stack_specific(slot - self.first.size(), amount)
                .await
        } else {
            self.first.remove_stack_specific(slot, amount).await
        }
    }

    fn get_max_count_per_stack(&self) -> u8 {
        self.first.get_max_count_per_stack()
    }

    async fn set_stack(&self, slot: usize, stack: ItemStack) {
        if slot >= self.first.size() {
            self.second.set_stack(slot - self.first.size(), stack).await
        } else {
            self.first.set_stack(slot, stack).await
        }
    }

    fn mark_dirty(&self) {
        self.first.mark_dirty();
        self.second.mark_dirty();
    }

    async fn on_open(&self) {
        self.first.on_open().await;
        self.second.on_open().await;
    }

    async fn on_close(&self) {
        self.first.on_close().await;
        self.second.on_close().await;
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

#[async_trait]
impl Clearable for DoubleInventory {
    async fn clear(&self) {
        self.first.clear().await;
        self.second.clear().await;
    }
}
