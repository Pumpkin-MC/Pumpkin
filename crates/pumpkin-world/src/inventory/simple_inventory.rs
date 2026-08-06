use crate::inventory::{Clearable, Inventory, InventoryFuture, split_stack};
use pumpkin_data::item_stack::ItemStack;
use std::any::Any;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct SimpleInventory {
    pub stacks: Vec<Arc<Mutex<ItemStack>>>,
}

impl SimpleInventory {
    #[must_use]
    pub fn new(size: usize) -> Self {
        Self {
            stacks: (0..size)
                .map(|_| Arc::new(Mutex::new(ItemStack::EMPTY.clone())))
                .collect(),
        }
    }
}

impl Clearable for SimpleInventory {
    fn clear(&self) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            for stack in &self.stacks {
                *stack.lock().await = ItemStack::EMPTY.clone();
            }
        })
    }
}

impl Inventory for SimpleInventory {
    fn size(&self) -> usize {
        self.stacks.len()
    }

    fn is_empty(&self) -> InventoryFuture<'_, bool> {
        Box::pin(async move {
            for stack in &self.stacks {
                if !stack.lock().await.is_empty() {
                    return false;
                }
            }
            true
        })
    }

    fn get_stack(&self, slot: usize) -> InventoryFuture<'_, Arc<Mutex<ItemStack>>> {
        Box::pin(async move { self.stacks[slot].clone() })
    }

    fn remove_stack(&self, slot: usize) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            let mut stack = self.stacks[slot].lock().await;
            let result = stack.clone();
            *stack = ItemStack::EMPTY.clone();
            result
        })
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move { split_stack(&self.stacks, slot, amount).await })
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            *self.stacks[slot].lock().await = stack;
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
