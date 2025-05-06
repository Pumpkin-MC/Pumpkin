use std::sync::{Arc, atomic::AtomicU8};

use async_trait::async_trait;
use pumpkin_world::inventory::Inventory;

use crate::{
    screen_handler::ScreenHandler,
    slot::{NormalSlot, Slot},
};

use super::recipes::{RecipeFinderScreenHandler, RecipeInputInventory};

// TODO: Implement ResultSlot
// CraftingResultSlot.java
#[derive(Debug)]
pub struct ResultSlot {
    pub inventory: Arc<dyn Inventory>,
    pub index: usize,
    pub id: AtomicU8,
}

impl ResultSlot {
    pub fn new(inventory: Arc<dyn Inventory>, index: usize) -> Self {
        Self {
            inventory,
            index,
            id: AtomicU8::new(0),
        }
    }
}
#[async_trait]
impl Slot for ResultSlot {
    fn get_inventory(&self) -> &Arc<dyn Inventory> {
        &self.inventory
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_id(&self, id: usize) {
        self.id
            .store(id as u8, std::sync::atomic::Ordering::Relaxed);
    }

    async fn mark_dirty(&self) {
        self.inventory.mark_dirty();
    }
}

// AbstractCraftingScreenHandler.java
#[async_trait]
pub trait CraftingScreenHandler<I: RecipeInputInventory>:
    RecipeFinderScreenHandler + ScreenHandler
{
    async fn add_result_slot(&mut self, crafing_inventory: &Arc<dyn RecipeInputInventory>) {
        let result_slot = ResultSlot::new(crafing_inventory.clone(), 0);
        self.add_slot(Arc::new(result_slot));
    }

    async fn add_input_slots(&mut self, crafing_inventory: &Arc<dyn RecipeInputInventory>) {
        let width = crafing_inventory.get_width();
        let height = crafing_inventory.get_height();
        for i in 0..width {
            for j in 0..height {
                let input_slot = NormalSlot::new(crafing_inventory.clone(), j + i * width);
                self.add_slot(Arc::new(input_slot));
            }
        }
    }
}
