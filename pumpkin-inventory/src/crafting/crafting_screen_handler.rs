use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::{
    screen_handler::ScreenHandler,
    slot::{NormalSlot, Slot},
};

use super::recipies::{RecipeFinderScreenHandler, RecipeInputInventory};

// TODO: Implement ResultSlot
// CraftingResultSlot.java
pub struct ResultSlot<I: RecipeInputInventory> {
    pub inventory: Arc<Mutex<I>>,
    pub index: usize,
    pub id: usize,
}

impl<I: RecipeInputInventory> ResultSlot<I> {
    pub fn new(inventory: Arc<Mutex<I>>, index: usize) -> Self {
        Self {
            inventory,
            index,
            id: 0,
        }
    }
}
#[async_trait]
impl<I: RecipeInputInventory> Slot<I> for ResultSlot<I> {
    fn get_inventory(&self) -> &Arc<Mutex<I>> {
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
    async fn add_result_slot(&mut self, crafing_inventory: &Arc<Mutex<I>>) {
        let result_slot = ResultSlot::new(crafing_inventory.clone(), 0);
        self.add_slot(result_slot);
    }

    async fn add_input_slots(&mut self, crafing_inventory: &Arc<Mutex<I>>) {
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
