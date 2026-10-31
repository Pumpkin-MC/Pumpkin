use crate::inventory::{Clearable, Inventory, InventoryHolder};
use pumpkin_data::item_stack::ItemStack;
use std::any::Any;
use std::sync::{Arc, RwLock};

pub struct SimpleInventory {
    pub stacks: RwLock<Vec<ItemStack>>,
    size: usize,
    holder: Option<Arc<dyn InventoryHolder>>,
}

impl SimpleInventory {
    #[must_use]
    pub fn new(size: usize) -> Self {
        Self {
            stacks: RwLock::new(vec![ItemStack::EMPTY.clone(); size]),
            size,
            holder: None,
        }
    }

    /// Associates an owner with this inventory before it is shared.
    #[must_use]
    pub fn with_holder(mut self, holder: Arc<dyn InventoryHolder>) -> Self {
        self.holder = Some(holder);
        self
    }
}

impl Clearable for SimpleInventory {
    fn clear(&self) {
        let mut stacks = self
            .stacks
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        stacks.fill_with(|| ItemStack::EMPTY.clone());
    }
}

impl Inventory for SimpleInventory {
    fn holder(&self) -> Option<Arc<dyn InventoryHolder>> {
        self.holder.clone()
    }

    fn size(&self) -> usize {
        self.size
    }

    fn is_empty(&self) -> bool {
        let stacks = self
            .stacks
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        stacks.iter().all(ItemStack::is_empty)
    }

    fn get_stack(&self, slot: usize) -> ItemStack {
        let stacks = self
            .stacks
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        stacks
            .get(slot)
            .cloned()
            .unwrap_or_else(|| ItemStack::EMPTY.clone())
    }

    fn remove_stack(&self, slot: usize) -> ItemStack {
        let mut stacks = self
            .stacks
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if slot < stacks.len() {
            std::mem::replace(&mut stacks[slot], ItemStack::EMPTY.clone())
        } else {
            ItemStack::EMPTY.clone()
        }
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> ItemStack {
        let mut stacks = self
            .stacks
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if slot < stacks.len() && !stacks[slot].is_empty() && amount > 0 {
            stacks[slot].split(amount)
        } else {
            ItemStack::EMPTY.clone()
        }
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) {
        let mut stacks = self
            .stacks
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if slot < stacks.len() {
            stacks[slot] = stack;
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
