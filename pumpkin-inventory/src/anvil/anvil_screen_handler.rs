use std::any::Any;
use std::sync::Arc;

use pumpkin_data::{item_stack::ItemStack, screen::WindowType};
use pumpkin_world::inventory::Inventory;

use crate::{
    player::player_inventory::PlayerInventory,
    screen_handler::{
        InventoryPlayer, ItemStackFuture, ScreenHandler, ScreenHandlerBehaviour,
        ScreenHandlerFuture, offer_or_drop_stack,
    },
    slot::NormalSlot,
    window_property::{Anvil, WindowProperty},
};

pub struct AnvilScreenHandler {
    pub inventory: Arc<dyn Inventory>,
    behaviour: ScreenHandlerBehaviour,
    pub rename_text: String,
    pub repair_cost: i16,
}

impl AnvilScreenHandler {
    #[expect(clippy::needless_pass_by_value)]
    pub fn new(
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        inventory: Arc<dyn Inventory>,
    ) -> Self {
        let mut handler = Self {
            inventory: inventory.clone(),
            behaviour: ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Anvil)),
            rename_text: String::new(),
            repair_cost: 0,
        };

        // Anvil specific slots: 2 input, 1 output
        for i in 0..3 {
            handler.add_slot(Arc::new(NormalSlot::new(inventory.clone(), i)));
        }

        let player_inventory: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&player_inventory);

        handler
    }

    pub async fn update_item_name(&mut self, name: String) {
        self.rename_text = name;
        self.update_result_slot().await;
        self.send_content_updates().await;
    }

    pub async fn update_result_slot(&mut self) {
        let input_a = {
            let lock = self.inventory.get_stack(0).await;
            lock.lock().await.clone()
        };

        if input_a.is_empty() {
            self.inventory.set_stack(2, ItemStack::EMPTY.clone()).await;
            self.set_repair_cost(0).await;
            return;
        }

        let mut result_item = input_a.clone();
        let mut cost = 0;

        // Basic renaming logic for now
        if !self.rename_text.is_empty() {
            result_item.set_custom_name(self.rename_text.clone());
            cost += 1;
        }

        // If combining with another item... we'll skip complex anvil logic for now
        // and just support renaming.
        if cost > 0 {
            self.inventory.set_stack(2, result_item).await;
            self.set_repair_cost(cost).await;
        } else {
            self.inventory.set_stack(2, ItemStack::EMPTY.clone()).await;
            self.set_repair_cost(0).await;
        }
    }

    pub async fn set_repair_cost(&mut self, cost: i16) {
        self.repair_cost = cost;
        if let Some(sync_handler) = self.behaviour.sync_handler.as_ref() {
            let (property_id, property_value) =
                WindowProperty::new(Anvil::RepairCost, cost).into_tuple();
            sync_handler
                .update_property(&self.behaviour, property_id as i32, property_value as i32)
                .await;
        }
    }
}

impl ScreenHandler for AnvilScreenHandler {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_behaviour(&self) -> &ScreenHandlerBehaviour {
        &self.behaviour
    }

    fn get_behaviour_mut(&mut self) -> &mut ScreenHandlerBehaviour {
        &mut self.behaviour
    }

    fn on_closed<'a>(&'a mut self, player: &'a dyn InventoryPlayer) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            self.default_on_closed(player).await;
            self.inventory.on_close().await;
            // Drop inputs from anvil
            for i in 0..2 {
                let stack = self.inventory.remove_stack(i).await;
                if !stack.is_empty() {
                    offer_or_drop_stack(player, stack).await;
                }
            }
            self.inventory.set_stack(2, ItemStack::EMPTY.clone()).await;
        })
    }

    fn quick_move<'a>(
        &'a mut self,
        _player: &'a dyn InventoryPlayer,
        slot_index: i32,
    ) -> ItemStackFuture<'a> {
        Box::pin(async move {
            let mut stack_left = ItemStack::EMPTY.clone();
            let slot = self.get_behaviour().slots[slot_index as usize].clone();

            if slot.has_stack().await {
                let slot_stack_lock = slot.get_stack().await;
                let slot_stack_guard = slot_stack_lock.lock().await;
                stack_left = slot_stack_guard.clone();
                drop(slot_stack_guard);

                let mut slot_stack_mut = slot_stack_lock.lock().await;

                if slot_index < 3 {
                    // From anvil to player
                    if !self
                        .insert_item(
                            &mut slot_stack_mut,
                            3,
                            self.get_behaviour().slots.len() as i32,
                            true,
                        )
                        .await
                    {
                        return ItemStack::EMPTY.clone();
                    }
                } else {
                    // From player to anvil input 0 and 1
                    if !self.insert_item(&mut slot_stack_mut, 0, 2, false).await {
                        return ItemStack::EMPTY.clone();
                    }
                }

                if slot_stack_mut.is_empty() {
                    drop(slot_stack_mut);
                    slot.set_stack(ItemStack::EMPTY.clone()).await;
                } else {
                    drop(slot_stack_mut);
                    slot.mark_dirty().await;
                }
            }

            stack_left
        })
    }

    fn on_slot_click<'a>(
        &'a mut self,
        slot_index: i32,
        button: i32,
        action_type: pumpkin_protocol::java::server::play::SlotActionType,
        player: &'a dyn InventoryPlayer,
    ) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            if slot_index == 2 {
                // Taking from output slot
                let result_slot = self.get_behaviour().slots[2].clone();
                if result_slot.has_stack().await {
                    let result_stack = result_slot.get_cloned_stack().await;
                    if !result_stack.is_empty() {
                        if player.experience_level() >= self.repair_cost as i32
                            || player.is_creative()
                        {
                            // Consume experience
                            if !player.is_creative() {
                                player
                                    .add_experience_levels(-(self.repair_cost as i32))
                                    .await;
                            }

                            // Consume inputs
                            let input_a = self.inventory.get_stack(0).await;
                            let mut input_a = input_a.lock().await;
                            input_a.decrement(1); // Usually consumes 1
                            if input_a.is_empty() {
                                *input_a = ItemStack::EMPTY.clone();
                            }
                            drop(input_a);
                            self.get_behaviour().slots[0].mark_dirty().await;
                        } else {
                            // Cancel click
                            self.send_content_updates().await;
                            return;
                        }
                    }
                }
            }

            self.internal_on_slot_click(slot_index, button, action_type, player)
                .await;
            if slot_index == 0 || slot_index == 1 || slot_index == 2 {
                self.update_result_slot().await;
                self.send_content_updates().await;
            }
        })
    }
}
