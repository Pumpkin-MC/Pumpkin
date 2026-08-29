use std::any::Any;
use std::sync::Arc;

use pumpkin_data::{item_stack::ItemStack, screen::WindowType};
use pumpkin_world::inventory::Inventory;

use crate::{
    player::player_inventory::PlayerInventory,
    screen_handler::{InventoryPlayer, ScreenHandler, ScreenHandlerBehaviour, offer_or_drop_stack},
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

    pub fn update_item_name(&mut self, name: String) {
        self.rename_text = name;
        self.update_result_slot();
        self.send_content_updates();
    }

    pub fn update_result_slot(&mut self) {
        let input_a = self.inventory.get_stack(0);

        if input_a.is_empty() {
            self.inventory.set_stack(2, ItemStack::EMPTY.clone());
            self.set_repair_cost(0);
            return;
        }

        let mut result_item = input_a;
        let mut cost = 0;

        // Basic renaming logic for now
        if !self.rename_text.is_empty() {
            result_item.set_custom_name(self.rename_text.clone());
            cost += 1;
        }

        // If combining with another item... we'll skip complex anvil logic for now
        // and just support renaming.
        if cost > 0 {
            self.inventory.set_stack(2, result_item);
            self.set_repair_cost(cost);
        } else {
            self.inventory.set_stack(2, ItemStack::EMPTY.clone());
            self.set_repair_cost(0);
        }
    }

    pub fn set_repair_cost(&mut self, cost: i16) {
        self.repair_cost = cost;
        if let Some(sync_handler) = self.behaviour.sync_handler.as_ref() {
            let (property_id, property_value) =
                WindowProperty::new(Anvil::RepairCost, cost).into_tuple();
            sync_handler.update_property(
                &self.behaviour,
                property_id as i32,
                property_value as i32,
            );
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

    fn on_closed(&mut self, player: &dyn InventoryPlayer) {
        self.default_on_closed(player);
        self.inventory.on_close();
        // Drop inputs from anvil
        for i in 0..2 {
            let stack = self.inventory.remove_stack(i);
            if !stack.is_empty() {
                offer_or_drop_stack(player, stack);
            }
        }
        self.inventory.set_stack(2, ItemStack::EMPTY.clone());
    }

    fn quick_move(&mut self, player: &dyn InventoryPlayer, slot_index: i32) -> ItemStack {
        let mut stack_left = ItemStack::EMPTY.clone();
        let slot = self.get_behaviour().slots[slot_index as usize].clone();

        if slot.has_stack() {
            let mut slot_stack = slot.get_stack();
            stack_left = slot_stack.clone();

            if slot_index < 3 {
                // From anvil to player
                if !self.insert_item(&mut slot_stack, 3, 39, true) {
                    return ItemStack::EMPTY.clone();
                }
                slot.on_quick_move_crafted(slot_stack.clone(), stack_left.clone());
            } else {
                // From player to anvil
                if !self.insert_item(&mut slot_stack, 0, 2, false) {
                    return ItemStack::EMPTY.clone();
                }
            }

            if slot_stack.item_count == stack_left.item_count {
                return ItemStack::EMPTY.clone();
            }

            slot.set_stack_prev(slot_stack.clone(), stack_left.clone());
            slot.on_take_item(player, &slot_stack);
            slot.mark_dirty();
        }

        stack_left
    }

    fn on_slot_click(
        &mut self,
        slot_index: i32,
        button: i32,
        action_type: pumpkin_protocol::java::server::play::SlotActionType,
        player: &dyn InventoryPlayer,
    ) {
        if slot_index == 2 {
            // Taking from output slot
            let result_slot = self.get_behaviour().slots[2].clone();
            if result_slot.has_stack() {
                let result_stack = result_slot.get_cloned_stack();
                if !result_stack.is_empty() {
                    if player.experience_level() >= self.repair_cost as i32 || player.is_creative()
                    {
                        // Consume experience
                        if !player.is_creative() {
                            player.add_experience_levels(-(self.repair_cost as i32));
                        }

                        // Consume inputs
                        self.inventory.set_stack(0, ItemStack::EMPTY.clone());
                        self.get_behaviour().slots[0].mark_dirty();
                    } else {
                        // Cancel click
                        self.send_content_updates();
                        return;
                    }
                }
            }
        }

        self.internal_on_slot_click(slot_index, button, action_type, player);
        if slot_index == 0 || slot_index == 1 || slot_index == 2 {
            self.update_result_slot();
            self.send_content_updates();
        }
    }
}
