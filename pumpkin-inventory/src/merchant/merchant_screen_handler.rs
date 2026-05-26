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
};

pub struct MerchantScreenHandler {
    pub inventory: Arc<dyn Inventory>,
    behaviour: ScreenHandlerBehaviour,
    selected_offer: usize,
    pub offers: Vec<pumpkin_protocol::java::client::play::MerchantOffer>,
}

impl MerchantScreenHandler {
    pub async fn new(
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        inventory: Arc<dyn Inventory>,
        offers: Vec<pumpkin_protocol::java::client::play::MerchantOffer>,
    ) -> Self {
        let mut handler = Self {
            inventory: inventory.clone(),
            behaviour: ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Merchant)),
            selected_offer: 0,
            offers,
        };

        inventory.on_open().await;

        // Merchant specific slots: 2 input, 1 output
        for i in 0..3 {
            handler.add_slot(Arc::new(NormalSlot::new(inventory.clone(), i)));
        }

        let player_inventory: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&player_inventory);

        handler
    }

    pub async fn set_selected_offer(&mut self, index: usize) {
        self.selected_offer = index;
        self.update_result_slot().await;
        self.send_content_updates().await;
    }

    pub async fn update_result_slot(&mut self) {
        if self.selected_offer >= self.offers.len() {
            self.inventory.set_stack(2, ItemStack::EMPTY.clone()).await;
            return;
        }

        let offer = &self.offers[self.selected_offer];
        let input_a = self.inventory.get_stack(0).await;
        let input_a = input_a.lock().await;
        let input_b = self.inventory.get_stack(1).await;
        let input_b = input_b.lock().await;

        let match_a = input_a.are_items_and_components_equal(&offer.base_cost_a.0)
            && input_a.item_count >= offer.base_cost_a.0.item_count;

        let match_b = offer.cost_b.as_ref().map_or_else(
            || input_b.is_empty(),
            |cost_b| {
                input_b.are_items_and_components_equal(&cost_b.0)
                    && input_b.item_count >= cost_b.0.item_count
            },
        );

        if match_a && match_b {
            self.inventory.set_stack(2, (*offer.output.0).clone()).await;
        } else {
            self.inventory.set_stack(2, ItemStack::EMPTY.clone()).await;
        }
    }
}

impl ScreenHandler for MerchantScreenHandler {
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
            // Vanilla drops items from merchant container on close
            for i in 0..2 {
                // Drop inputs only, output is virtual/ghost in some sense or just cleared
                let stack = self.inventory.remove_stack(i).await;
                if !stack.is_empty() {
                    offer_or_drop_stack(player, stack).await;
                }
            }
            // Clear output slot
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
                    // From merchant slots to player inventory
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
                    // From player inventory to merchant inputs (0 and 1)
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
                // Special handling for taking from output slot
                let result_slot = self.get_behaviour().slots[2].clone();
                if result_slot.has_stack().await {
                    let result_stack = result_slot.get_cloned_stack().await;
                    if !result_stack.is_empty() {
                        // Consume inputs
                        let offer = &self.offers[self.selected_offer];

                        let input_a = self.inventory.get_stack(0).await;
                        let mut input_a = input_a.lock().await;
                        input_a.decrement(offer.base_cost_a.0.item_count);
                        if input_a.is_empty() {
                            *input_a = ItemStack::EMPTY.clone();
                        }
                        drop(input_a);
                        self.get_behaviour().slots[0].mark_dirty().await;

                        if let Some(cost_b) = &offer.cost_b {
                            let input_b = self.inventory.get_stack(1).await;
                            let mut input_b = input_b.lock().await;
                            input_b.decrement(cost_b.0.item_count);
                            if input_b.is_empty() {
                                *input_b = ItemStack::EMPTY.clone();
                            }
                            drop(input_b);
                            self.get_behaviour().slots[1].mark_dirty().await;
                        }

                        // Award XP
                        player.award_experience(offer.xp).await;
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
