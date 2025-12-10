use std::{any::Any, pin::Pin, sync::Arc};

use pumpkin_world::{
    inventory::{Clearable, Inventory, InventoryFuture},
    item::ItemStack,
};
use tokio::sync::Mutex;

pub struct DoubleInventory {
    first: Arc<dyn Inventory>,
    second: Arc<dyn Inventory>,
}

impl DoubleInventory {
    pub fn new(first: Arc<dyn Inventory>, second: Arc<dyn Inventory>) -> Arc<Self> {
        Arc::new(Self { first, second })
    }
}

impl Inventory for DoubleInventory {
    fn size(&self) -> usize {
        self.first.size() + self.second.size()
    }

    fn is_empty(&self) -> InventoryFuture<'_, bool> {
        Box::pin(async move { self.first.is_empty().await && self.second.is_empty().await })
    }

    fn get_stack(&self, slot: usize) -> InventoryFuture<'_, Arc<Mutex<ItemStack>>> {
        Box::pin(async move {
            if slot >= self.first.size() {
                self.second.get_stack(slot - self.first.size()).await
            } else {
                self.first.get_stack(slot).await
            }
        })
    }

    fn remove_stack(&self, slot: usize) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            if slot >= self.first.size() {
                self.second.remove_stack(slot - self.first.size()).await
            } else {
                self.first.remove_stack(slot).await
            }
        })
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            if slot >= self.first.size() {
                self.second
                    .remove_stack_specific(slot - self.first.size(), amount)
                    .await
            } else {
                self.first.remove_stack_specific(slot, amount).await
            }
        })
    }

    fn get_max_count_per_stack(&self) -> u8 {
        self.first.get_max_count_per_stack()
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            if slot >= self.first.size() {
                self.second.set_stack(slot - self.first.size(), stack).await
            } else {
                self.first.set_stack(slot, stack).await
            }
        })
    }

    fn mark_dirty(&self) {
        self.first.mark_dirty();
        self.second.mark_dirty();
    }

    fn on_open(&self) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            self.first.on_open().await;
            self.second.on_open().await;
        })
    }

    fn on_close(&self) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            self.first.on_close().await;
            self.second.on_close().await;
        })
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
    fn clear(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            self.first.clear().await;
            self.second.clear().await;
        })
    }
}
