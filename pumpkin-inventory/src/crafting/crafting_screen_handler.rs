use std::sync::Arc;
use std::sync::atomic::AtomicU8;

use async_trait::async_trait;

use pumpkin_world::inventory::Inventory;
use pumpkin_world::item::ItemStack;
use tokio::sync::Mutex;

use crate::screen_handler::{
    InventoryPlayer, ScreenHandler, ScreenHandlerBehaviour, ScreenHandlerListener,
};
use crate::slot::{NormalSlot, Slot};

use super::recipes::RecipeInputInventory;

#[derive(Debug)]
pub struct ResultSlot {
    pub inventory: Arc<dyn RecipeInputInventory>,
    pub amount: AtomicU8,
    pub id: AtomicU8,
    pub result: Arc<Mutex<ItemStack>>,
}

impl ResultSlot {
    pub fn new(inventory: Arc<dyn RecipeInputInventory>) -> Self {
        Self {
            inventory,
            amount: AtomicU8::new(0),
            id: AtomicU8::new(0),
            result: Arc::new(Mutex::new(ItemStack::EMPTY)),
        }
    }
}

#[async_trait]
impl Slot for ResultSlot {
    async fn can_insert(&self, _stack: &ItemStack) -> bool {
        false
    }

    async fn take_stack(&self, amount: u8) -> ItemStack {
        // self.amount.fetch_add(
        //     amount.min(self.get_stack().await.lock().await.item_count),
        //     Ordering::Relaxed,
        // );

        if self.has_stack().await {
            let stack = *self.result.lock().await;
            self.on_crafted(stack, amount).await;
            stack.copy_with_count(amount)
        } else {
            ItemStack::EMPTY
        }
    }

    async fn on_crafted(&self, _stack: ItemStack, amount: u8) {
        // self.amount.fetch_add(amount, Ordering::Relaxed);

        // if self.amount.load(Ordering::Relaxed) > 0 {
        //     // stack.on_craft
        // }

        // self.amount.store(0, Ordering::Relaxed);
        // TODO: Unlock recipes for the recipe book

        for i in 0..self.inventory.size() {
            let slot = self.inventory.get_stack(i).await;
            let mut slot = slot.lock().await;
            if !slot.is_empty() {
                slot.item_count -= amount;
            }
        }
    }

    async fn on_take_item(&self, _player: &dyn InventoryPlayer, stack: &ItemStack) {
        self.on_crafted(*stack, stack.item_count).await;
        self.mark_dirty().await;
    }

    async fn get_max_item_count(&self) -> u8 {
        let mut count = u8::MAX;
        for i in 0..self.inventory.size() {
            let slot = self.inventory.get_stack(i).await;
            let slot = slot.lock().await;
            if !slot.is_empty() {
                count = count.min(slot.item_count);
            }
        }
        count
    }

    async fn has_stack(&self) -> bool {
        !self.result.lock().await.is_empty()
    }

    async fn get_stack(&self) -> Arc<Mutex<ItemStack>> {
        self.result.clone()
    }

    async fn get_cloned_stack(&self) -> ItemStack {
        *self.result.lock().await
    }

    fn get_inventory(&self) -> Arc<dyn Inventory> {
        self.inventory.clone()
    }

    fn get_index(&self) -> usize {
        0
    }

    fn set_id(&self, id: usize) {
        self.id
            .store(id as u8, std::sync::atomic::Ordering::Relaxed);
    }

    async fn mark_dirty(&self) {
        self.inventory.mark_dirty();
    }
}

#[async_trait]
impl ScreenHandlerListener for ResultSlot {
    async fn on_slot_update(
        &self,
        screen_handler: &ScreenHandlerBehaviour,
        slot: u8,
        _stack: ItemStack,
    ) {
        if (0..=(self.inventory.get_width() * self.inventory.get_height()))
            .contains(&(slot as usize))
        {
            let result = self
                .inventory
                .match_recipe()
                .await
                .map(ItemStack::from)
                .unwrap_or(ItemStack::EMPTY);
            *self.result.lock().await = result;

            let next_revision = screen_handler.next_revision();
            if let Some(sync_handler) = screen_handler.sync_handler.as_ref() {
                sync_handler
                    .update_slot(screen_handler, 0, &result, next_revision)
                    .await;
            }
        }
    }
}

#[async_trait]
pub trait CraftingScreenHandler<I: RecipeInputInventory>: ScreenHandler {
    async fn add_recipe_slots(&mut self, crafing_inventory: Arc<dyn RecipeInputInventory>) {
        let result_slot = Arc::new(ResultSlot::new(crafing_inventory.clone()));
        self.add_slot(result_slot.clone());

        let width = crafing_inventory.get_width();
        let height = crafing_inventory.get_height();
        for i in 0..width {
            for j in 0..height {
                let input_slot = NormalSlot::new(crafing_inventory.clone(), j + i * width);
                self.add_slot(Arc::new(input_slot));
            }
        }

        self.add_listener(result_slot).await;
    }
}
