use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_world::item::ItemStack;
use tokio::sync::Mutex;

use crate::{
    inventory::Inventory,
    screen_handler::ScreenHandler,
    slot::{NormalSlot, Slot},
};

use super::recipies::RecipeInputInventory;

#[derive(Debug, Clone)]
pub struct CraftingInventory {
    pub width: u8,
    pub height: u8,
}

impl Inventory for CraftingInventory {
    fn size(&self) -> usize {
        todo!()
    }

    fn is_empty(&self) -> bool {
        todo!()
    }

    fn get_stack(&mut self, slot: usize) -> &mut ItemStack {
        todo!()
    }

    fn get_stack_ref(&self, slot: usize) -> &ItemStack {
        todo!()
    }

    fn remove_stack_specific(&mut self, slot: usize, amount: u8) -> ItemStack {
        todo!()
    }

    fn remove_stack(&mut self, slot: usize) -> ItemStack {
        todo!()
    }

    fn set_stack(&mut self, slot: usize, stack: ItemStack) {
        todo!()
    }

    fn mark_dirty(&mut self) {
        todo!()
    }

    fn clone_box(&self) -> Box<dyn Inventory> {
        Box::new(self.clone())
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
