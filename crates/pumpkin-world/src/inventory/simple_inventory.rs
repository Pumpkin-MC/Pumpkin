use crate::inventory::{Clearable, Inventory, InventoryFuture};
use pumpkin_data::item_stack::ItemStack;
use std::any::Any;
use tokio::sync::RwLock;

pub struct SimpleInventory {
    pub stacks: RwLock<Vec<ItemStack>>,
    size: usize,
}

impl SimpleInventory {
    #[must_use]
    pub fn new(size: usize) -> Self {
        Self {
            stacks: RwLock::new(vec![ItemStack::EMPTY.clone(); size]),
            size,
        }
    }
}

impl Clearable for SimpleInventory {
    fn clear(&self) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            let mut stacks = self.stacks.write().await;
            stacks.fill_with(|| ItemStack::EMPTY.clone());
        })
    }
}

impl Inventory for SimpleInventory {
    fn size(&self) -> usize {
        self.size
    }

    fn is_empty(&self) -> InventoryFuture<'_, bool> {
        Box::pin(async move {
            let stacks = self.stacks.read().await;
            stacks.iter().all(ItemStack::is_empty)
        })
    }

    fn get_stack(&self, slot: usize) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            let stacks = self.stacks.read().await;
            stacks
                .get(slot)
                .cloned()
                .unwrap_or_else(|| ItemStack::EMPTY.clone())
        })
    }

    fn remove_stack(&self, slot: usize) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            let mut stacks = self.stacks.write().await;
            if slot < stacks.len() {
                std::mem::replace(&mut stacks[slot], ItemStack::EMPTY.clone())
            } else {
                ItemStack::EMPTY.clone()
            }
        })
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            let mut stacks = self.stacks.write().await;
            if slot < stacks.len() && !stacks[slot].is_empty() && amount > 0 {
                stacks[slot].split(amount)
            } else {
                ItemStack::EMPTY.clone()
            }
        })
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            let mut stacks = self.stacks.write().await;
            if slot < stacks.len() {
                stacks[slot] = stack;
            }
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
