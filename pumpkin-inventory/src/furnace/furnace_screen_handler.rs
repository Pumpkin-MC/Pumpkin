use std::{any::Any, pin::Pin, sync::Arc};

use pumpkin_data::fuels::is_fuel;
use pumpkin_world::{block::entities::BlockEntity, inventory::Inventory, item::ItemStack};

use crate::{
    player::player_inventory::PlayerInventory,
    screen_handler::{
        InventoryPlayer, ItemStackFuture, ScreenHandler, ScreenHandlerBehaviour,
        ScreenHandlerFuture, ScreenHandlerListener, ScreenProperty,
    },
};

use super::furnace_slot::{FurnaceSlot, FurnaceSlotType};

pub struct FurnaceScreenHandler {
    pub inventory: Arc<dyn Inventory>,
    behaviour: ScreenHandlerBehaviour,
}

impl FurnaceScreenHandler {
    pub async fn new(
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        inventory: Arc<dyn Inventory>,
        furnace_block_entity: Arc<dyn BlockEntity>,
    ) -> Self {
        let furnace_property_delegate = furnace_block_entity.to_property_delegate().unwrap();
        let mut handler = Self {
            inventory,
            behaviour: ScreenHandlerBehaviour::new(
                sync_id,
                Some(pumpkin_data::screen::WindowType::Furnace),
            ),
        };

        struct FurnaceScreenListener;
        impl ScreenHandlerListener for FurnaceScreenListener {
            fn on_property_update<'a>(
                &'a self,
                screen_handler: &'a ScreenHandlerBehaviour,
                property: u8,
                value: i32,
            ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
                Box::pin(async move {
                    if let Some(sync_handler) = screen_handler.sync_handler.as_ref() {
                        sync_handler
                            .update_property(screen_handler, property as i32, value)
                            .await;
                    }
                })
            }
        }

        // 0: Fire icon (fuel left) counting from fuel burn time down to 0 (in-game ticks)
        // 1: Maximum fuel burn time fuel burn time or 0 (in-game ticks)
        // 2: Progress arrow counting from 0 to maximum progress (in-game ticks)
        // 3: Maximum progress always 200 on the vanilla server
        for i in 0..4 {
            handler.add_property(ScreenProperty::new(furnace_property_delegate.clone(), i));
        }

        handler.add_listener(Arc::new(FurnaceScreenListener)).await;
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

impl ScreenHandler for FurnaceScreenHandler {
    fn on_closed<'a>(&'a mut self, player: &'a dyn InventoryPlayer) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            self.default_on_closed(player).await;
            // TODO: self.inventory.on_closed(player).await;
        })
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

    fn quick_move<'a>(
        &'a mut self,
        _player: &'a dyn InventoryPlayer,
        slot_index: i32,
    ) -> ItemStackFuture<'a> {
        Box::pin(async move {
            const FUEL_SLOT: i32 = 1; // Note: Slots 0, 1, 2 are Furnace slots.

            let mut stack_left = ItemStack::EMPTY.clone();

            let slot = self.get_behaviour().slots[slot_index as usize].clone();

            if !slot.has_stack().await {
                return stack_left;
            }

            let slot_stack_lock = slot.get_stack().await;

            // Acquire the lock to read/clone the stack
            let mut stack = slot_stack_lock.lock().await;
            stack_left = stack.clone();

            let success = if slot_index < 3 {
                // If clicked slot is one of the Furnace slots (0, 1, 2):
                // Try to move to player inventory (slots 3 onwards, starting from the end)
                self.insert_item(&mut stack, 3, self.get_behaviour().slots.len() as i32, true)
                    .await
            } else if is_fuel(stack.item.id) {
                // If clicked slot is in the player inventory (3+) and contains fuel:
                // Try to move to the Furnace's Fuel slot (slot 1)
                self.insert_item(&mut stack, FUEL_SLOT, 3, false).await
            } else {
                // If clicked slot is in the player inventory (3+) and NOT fuel (must be a smeltable item):
                // Try to move to the Furnace's Input/Smelting slot (slot 0)
                self.insert_item(&mut stack, 0, 3, false).await
            };

            if !success {
                return ItemStack::EMPTY.clone();
            }

            if stack.is_empty() {
                drop(stack); // Release lock before awaiting
                slot.set_stack(ItemStack::EMPTY.clone()).await;
            } else {
                slot.mark_dirty().await;
            }

            stack_left
        })
    }
}
