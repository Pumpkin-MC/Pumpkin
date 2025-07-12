use std::{any::Any, sync::Arc};

use async_trait::async_trait;
use pumpkin_world::{
    block::entities::{BlockEntity, furnace::FurnaceBlockEntity},
    inventory::Inventory,
    item::ItemStack,
};

use crate::{
    player::player_inventory::PlayerInventory,
    screen_handler::{InventoryPlayer, ScreenHandler, ScreenHandlerBehaviour, ScreenProperty},
};

use super::furnace_slot::{FurnaceSlot, FurnaceSlotType};

pub struct FurnaceScreenHandler {
    pub inventory: Arc<dyn Inventory>,
    pub furnace_block_entity: Arc<dyn BlockEntity>,
    behaviour: ScreenHandlerBehaviour,
}

impl FurnaceScreenHandler {
    pub fn new(
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        inventory: Arc<dyn Inventory>,
        furnace_block_entity: Arc<dyn BlockEntity>,
    ) -> Self {
        let mut handler = Self {
            inventory,
            behaviour: ScreenHandlerBehaviour::new(
                sync_id,
                Some(pumpkin_data::screen::WindowType::Furnace),
            ),
            furnace_block_entity,
        };

        // 0: Fire icon (fuel left) counting from fuel burn time down to 0 (in-game ticks)
        // 1: Maximum fuel burn time fuel burn time or 0 (in-game ticks)
        // 2: Progress arrow counting from 0 to maximum progress (in-game ticks)
        // 3: Maximum progress always 200 on the vanilla server
        for _ in 0..4 {
            handler.add_property(ScreenProperty::new(0));
        }

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

    async fn sync_state(&mut self) {
        let furnace_block_entity = self
            .furnace_block_entity
            .as_any()
            .downcast_ref::<FurnaceBlockEntity>()
            .unwrap();
        let cook_progress = (furnace_block_entity.get_cook_progress().await * 200.0) as i32;
        let fuel_progress = (furnace_block_entity.get_fuel_progress().await * 200.0) as i32;

        let behaviour = self.get_behaviour_mut();

        behaviour.properties[0].set(fuel_progress);
        behaviour.properties[2].set(cook_progress);
        behaviour.properties[3].set(200);

        let mut previous_tracked_stacks = Vec::new();

        for i in 0..behaviour.slots.len() {
            let stack = behaviour.slots[i].get_cloned_stack().await;
            previous_tracked_stacks.push(stack);
            behaviour.previous_tracked_stacks[i].set_received_stack(stack);
        }

        let cursor_stack = *behaviour.cursor_stack.lock().await;
        behaviour
            .previous_cursor_stack
            .set_received_stack(cursor_stack);

        for i in 0..behaviour.properties.len() {
            let property_val = behaviour.properties[i].get();
            behaviour.tracked_property_values[i] = property_val;
        }

        let next_revision = behaviour.next_revision();

        if let Some(sync_handler) = behaviour.sync_handler.as_ref() {
            sync_handler
                .update_state(
                    behaviour,
                    &previous_tracked_stacks,
                    &cursor_stack,
                    behaviour.tracked_property_values.clone(),
                    next_revision,
                )
                .await;
        }
    }
}
