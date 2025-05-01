use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_world::item::ItemStack;
use tokio::sync::Mutex;

use crate::inventory::Inventory;

use super::recipies::RecipeInputInventory;

#[derive(Debug, Clone)]
pub struct CraftingInventory {
    pub width: u8,
    pub height: u8,
    pub slots: Vec<Arc<Mutex<ItemStack>>>,
}

impl CraftingInventory {
    pub fn new(width: u8, height: u8) -> Self {
        Self {
            width,
            height,
            slots: {
                // Creates a Vec with different Mutexes for each slot
                let mut v = Vec::with_capacity(width as usize * height as usize);
                (0..width as usize * height as usize)
                    .for_each(|_| v.push(Arc::new(Mutex::new(ItemStack::EMPTY))));
                v
            },
        }
    }
}

#[async_trait]
impl Inventory for CraftingInventory {
    fn size(&self) -> usize {
        self.slots.len()
    }

    async fn is_empty(&self) -> bool {
        todo!()
    }

    fn get_stack(&self, slot: usize) -> Arc<Mutex<ItemStack>> {
        self.slots[slot].clone()
    }

    async fn remove_stack(&mut self, slot: usize) -> ItemStack {
        todo!()
    }

    async fn remove_stack_specific(&self, slot: usize, amount: u8) -> ItemStack {
        todo!()
    }

    async fn set_stack(&mut self, slot: usize, stack: ItemStack) {
        todo!()
    }

    fn mark_dirty(&mut self) {
        todo!()
    }
}

impl RecipeInputInventory for CraftingInventory {
    fn get_width(&self) -> usize {
        self.width as usize
    }

    fn get_height(&self) -> usize {
        self.height as usize
    }
}
