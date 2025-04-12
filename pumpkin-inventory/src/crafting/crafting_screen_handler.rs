use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::{
    inventory::Inventory,
    screen_handler::ScreenHandler,
    slot::{NormalSlot, Slot},
};

use super::recipies::{RecipeFinderScreenHandler, RecipeInputInventory};

// TODO: Implement ResultSlot
// CraftingResultSlot.java
#[derive(Debug)]
pub struct ResultSlot {
    pub inventory: Arc<Mutex<dyn Inventory>>,
    pub index: usize,
    pub id: usize,
}

impl ResultSlot {
    pub fn new(inventory: Arc<Mutex<dyn Inventory>>, index: usize) -> Self {
        Self {
            inventory,
            index,
            id: 0,
        }
    }
}
#[async_trait]
impl Slot for ResultSlot {
    fn get_inventory(&self) -> &Arc<Mutex<dyn Inventory>> {
        &self.inventory
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_id(&mut self, id: usize) {
        self.id = id;
    }

    async fn mark_dirty(&self) {
        self.inventory.lock().await.mark_dirty();
    }
}

// AbstractCraftingScreenHandler.java
#[async_trait]
pub trait CraftingScreenHandler<I: RecipeInputInventory>:
    RecipeFinderScreenHandler + ScreenHandler
{
    async fn add_result_slot(&mut self, crafing_inventory: &Arc<Mutex<dyn RecipeInputInventory>>) {
        let result_slot = ResultSlot::new(crafing_inventory.clone(), 0);
        self.add_slot(result_slot);
    }

    async fn add_input_slots(&mut self, crafing_inventory: &Arc<Mutex<dyn RecipeInputInventory>>) {
        let crafting_temp = crafing_inventory.lock().await;
        let width = crafting_temp.get_width();
        let height = crafting_temp.get_height();
        drop(crafting_temp);
        for i in 0..width {
            for j in 0..height {
                let input_slot = NormalSlot::new(crafing_inventory.clone(), j + i * width);
                self.add_slot(input_slot);
            }
        }
    }
}
