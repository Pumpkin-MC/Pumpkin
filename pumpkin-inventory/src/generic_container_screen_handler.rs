use std::{any::Any, sync::Arc};

use pumpkin_data::screen::WindowType;
use pumpkin_world::{inventory::Inventory, item::ItemStack};

use crate::{
    player::player_inventory::PlayerInventory,
    screen_handler::{
        InventoryPlayer, ItemStackFuture, ScreenHandler, ScreenHandlerBehaviour,
        ScreenHandlerFuture,
    },
    slot::NormalSlot,
};

pub async fn create_generic_9x3(
    sync_id: u8,
    player_inventory: &Arc<PlayerInventory>,
    inventory: Arc<dyn Inventory>,
) -> GenericContainerScreenHandler {
    GenericContainerScreenHandler::new(
        WindowType::Generic9x3,
        sync_id,
        player_inventory,
        inventory,
        3,
        9,
    )
    .await
}

pub async fn create_generic_9x6(
    sync_id: u8,
    player_inventory: &Arc<PlayerInventory>,
    inventory: Arc<dyn Inventory>,
) -> GenericContainerScreenHandler {
    GenericContainerScreenHandler::new(
        WindowType::Generic9x6,
        sync_id,
        player_inventory,
        inventory,
        6,
        9,
    )
    .await
}

pub async fn create_generic_3x3(
    sync_id: u8,
    player_inventory: &Arc<PlayerInventory>,
    inventory: Arc<dyn Inventory>,
) -> GenericContainerScreenHandler {
    GenericContainerScreenHandler::new(
        WindowType::Generic3x3,
        sync_id,
        player_inventory,
        inventory,
        3,
        3,
    )
    .await
}

pub async fn create_hopper(
    sync_id: u8,
    player_inventory: &Arc<PlayerInventory>,
    inventory: Arc<dyn Inventory>,
) -> GenericContainerScreenHandler {
    GenericContainerScreenHandler::new(
        WindowType::Hopper,
        sync_id,
        player_inventory,
        inventory,
        1,
        5,
    )
    .await
}

pub struct GenericContainerScreenHandler {
    pub inventory: Arc<dyn Inventory>,
    pub rows: u8,
    pub columns: u8,
    behaviour: ScreenHandlerBehaviour,
}

impl GenericContainerScreenHandler {
    async fn new(
        screen_type: WindowType,
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        inventory: Arc<dyn Inventory>,
        rows: u8,
        columns: u8,
    ) -> Self {
        let mut handler = Self {
            inventory: inventory.clone(),
            rows,
            columns,
            behaviour: ScreenHandlerBehaviour::new(sync_id, Some(screen_type)),
        };

        // TODO: Add player entity as a parameter
        inventory.on_open().await;

        handler.add_inventory_slots();
        let player_inventory: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&player_inventory);

        handler
    }

    fn add_inventory_slots(&mut self) {
        for i in 0..self.rows {
            for j in 0..self.columns {
                self.add_slot(Arc::new(NormalSlot::new(
                    self.inventory.clone(),
                    (j + i * self.columns) as usize,
                )));
            }
        }
    }
}

impl ScreenHandler for GenericContainerScreenHandler {
    fn on_closed<'a>(&'a mut self, player: &'a dyn InventoryPlayer) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            self.default_on_closed(player).await;
            self.inventory.on_close().await;
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
            let mut stack_left = ItemStack::EMPTY.clone();
            // Assuming bounds check passed for slot_index by caller or within quick_move spec
            let slot = self.get_behaviour().slots[slot_index as usize].clone();

            if slot.has_stack().await {
                let slot_stack_lock = slot.get_stack().await;
                let slot_stack_guard = slot_stack_lock.lock().await;
                stack_left = slot_stack_guard.clone();
                // Release the guard before calling insert_item which needs its own lock
                drop(slot_stack_guard);

                // Re-acquire lock for insert_item (which expects &mut ItemStack)
                let mut slot_stack_mut = slot_stack_lock.lock().await;

                if slot_index < (self.rows * 9) as i32 {
                    // Move from inventory to player area (end)
                    if !self
                        .insert_item(
                            &mut slot_stack_mut,
                            (self.rows * 9).into(),
                            self.get_behaviour().slots.len() as i32,
                            true,
                        )
                        .await
                    {
                        return ItemStack::EMPTY.clone();
                    }
                } else if !self
                    .insert_item(&mut slot_stack_mut, 0, (self.rows * 9).into(), false)
                    .await
                {
                    // Move from player area to inventory (start)
                    return ItemStack::EMPTY.clone();
                }

                // Check the resulting state of the slot stack after insert_item
                if slot_stack_mut.is_empty() {
                    drop(slot_stack_mut); // Release lock
                    slot.set_stack(ItemStack::EMPTY.clone()).await;
                } else {
                    drop(slot_stack_mut); // Release lock
                    slot.mark_dirty().await;
                }
            }

            stack_left
        })
    }
}
