use std::{any::Any, sync::Arc};

use async_trait::async_trait;
use pumpkin_world::{inventory::Inventory, item::ItemStack};

use crate::{
    player::player_inventory::PlayerInventory,
    screen_handler::{InventoryPlayer, ScreenHandler, ScreenHandlerBehaviour},
};

use super::furnace_slot::{FurnaceSlot, FurnaceSlotType};

pub struct FurnaceScreenHandler {
    pub inventory: Arc<dyn Inventory>,
    behaviour: ScreenHandlerBehaviour,
}

impl FurnaceScreenHandler {
    pub fn new(
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        inventory: Arc<dyn Inventory>,
    ) -> Self {
        let mut handler = Self {
            inventory,
            behaviour: ScreenHandlerBehaviour::new(
                sync_id,
                Some(pumpkin_data::screen::WindowType::Furnace),
            ),
        };

        handler.add_inventory_slots();
        let player_inventory: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&player_inventory);

        handler
    }

    fn add_inventory_slots(&mut self) {
        self.add_slot(Arc::new(FurnaceSlot::new(
            self.inventory.clone(),
            FurnaceSlotType::Top,
        )));
        self.add_slot(Arc::new(FurnaceSlot::new(
            self.inventory.clone(),
            FurnaceSlotType::Bottom,
        )));
        self.add_slot(Arc::new(FurnaceSlot::new(
            self.inventory.clone(),
            FurnaceSlotType::Side,
        )));
    }
}

#[async_trait]
impl ScreenHandler for FurnaceScreenHandler {
    async fn on_closed(&mut self, player: &dyn InventoryPlayer) {
        self.default_on_closed(player).await;
        //TODO: self.inventory.on_closed(player).await;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_behaviour(&self) -> &ScreenHandlerBehaviour {
        &self.behaviour
    }

    fn get_behaviour_mut(&mut self) -> &mut ScreenHandlerBehaviour {
        &mut self.behaviour
    }

    async fn quick_move(&mut self, _player: &dyn InventoryPlayer, slot_index: i32) -> ItemStack {
        let mut stack_left = ItemStack::EMPTY;
        let slot = self.get_behaviour().slots[slot_index as usize].clone();

        if slot.has_stack().await {
            let slot_stack = slot.get_stack().await;
            stack_left = *slot_stack.lock().await;

            if slot_index < 3 {
                if !self
                    .insert_item(
                        &mut *slot_stack.lock().await,
                        3,
                        self.get_behaviour().slots.len() as i32,
                        true,
                    )
                    .await
                {
                    return ItemStack::EMPTY;
                }
            } else if !self
                .insert_item(&mut *slot_stack.lock().await, 0, 3, false)
                .await
            {
                return ItemStack::EMPTY;
            }

            if stack_left.is_empty() {
                slot.set_stack(ItemStack::EMPTY).await;
            } else {
                slot.mark_dirty().await;
            }
        }

        return stack_left;
    }
}
