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
    pub on_trade: Option<Box<dyn Fn(usize) + Send + Sync>>,
}

/// Which physical payment slot holds which trade ingredient. Vanilla lets the player
/// put either ingredient in either slot (`MerchantContainer.updateSellItem`,
/// `MerchantResultSlot.onTake`'s `offer.take(buyA, buyB) || offer.take(buyB, buyA)`).
enum SlotOrder {
    /// slot 0 matches `cost_a`, slot 1 matches `cost_b`.
    Normal,
    /// slot 1 matches `cost_a`, slot 0 matches `cost_b`.
    Swapped,
}

/// `MerchantOffer::getModifiedCostCount` (`MerchantOffer.java:123-127`) via
/// `modified_cost_count`: demand/special-price-adjusted `cost_a` count.
fn cost_a_count(offer: &pumpkin_protocol::java::client::play::MerchantOffer) -> i32 {
    pumpkin_protocol::java::client::play::MerchantOffer::modified_cost_count(
        offer.base_cost_a.0.item_count as i32,
        offer.demand,
        offer.price_multiplier,
        offer.special_price,
        offer.base_cost_a.0.get_max_stack_size() as i32,
    )
}

/// `MerchantOffer::satisfiedBy` (`MerchantOffer.java:213-219`): does `buy_a` satisfy
/// `cost_a` and `buy_b` satisfy `cost_b` (or is empty if there's no `cost_b`)?
fn offer_satisfied_by(
    offer: &pumpkin_protocol::java::client::play::MerchantOffer,
    buy_a: &ItemStack,
    buy_b: &ItemStack,
) -> bool {
    if !buy_a.are_items_and_components_equal(&offer.base_cost_a.0)
        || (buy_a.item_count as i32) < cost_a_count(offer)
    {
        return false;
    }
    offer.cost_b.as_ref().map_or_else(
        || buy_b.is_empty(),
        |cost_b| {
            buy_b.are_items_and_components_equal(&cost_b.0)
                && buy_b.item_count >= cost_b.0.item_count
        },
    )
}

/// Try both slot orders, matching `offer.take(buyA, buyB) || offer.take(buyB, buyA)`.
fn offer_matches(
    offer: &pumpkin_protocol::java::client::play::MerchantOffer,
    slot0: &ItemStack,
    slot1: &ItemStack,
) -> Option<SlotOrder> {
    if offer_satisfied_by(offer, slot0, slot1) {
        Some(SlotOrder::Normal)
    } else if offer_satisfied_by(offer, slot1, slot0) {
        Some(SlotOrder::Swapped)
    } else {
        None
    }
}

/// Maps a matched `SlotOrder` to (index of the slot holding `cost_a`, index of the slot
/// holding `cost_b`).
const fn payment_slot_indices(order: &SlotOrder) -> (usize, usize) {
    match order {
        SlotOrder::Normal => (0, 1),
        SlotOrder::Swapped => (1, 0),
    }
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
            on_trade: None,
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
        let slot0 = self.inventory.get_stack(0).await;
        let slot0 = slot0.lock().await;
        let slot1 = self.inventory.get_stack(1).await;
        let slot1 = slot1.lock().await;

        // The two payment slots can hold either ingredient in either order (see
        // `offer_matches` above), so both orders must be tried before deciding whether
        // the trade is affordable.
        if offer_matches(offer, &slot0, &slot1).is_some() {
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
                        // Figure out which physical slot holds cost_a and which holds
                        // cost_b before consuming -- the player may have put either
                        // ingredient in either slot (`offer_matches` above).
                        let order = {
                            let slot0 = self.inventory.get_stack(0).await;
                            let slot0 = slot0.lock().await;
                            let slot1 = self.inventory.get_stack(1).await;
                            let slot1 = slot1.lock().await;
                            offer_matches(&self.offers[self.selected_offer], &slot0, &slot1)
                        };

                        // `order` should always be `Some` here since the output slot is
                        // only populated by `update_result_slot` when a match exists. If
                        // it somehow isn't, mirror vanilla `MerchantOffer::take`: neither
                        // slot order was satisfied, so nothing is consumed.
                        if let Some(order) = order {
                            let (slot_a_index, slot_b_index) = payment_slot_indices(&order);

                            // Consume inputs. `MerchantOffer::getCostA`
                            // (`MerchantOffer.java:119-121`): charge the demand/special-
                            // price-modified count, matching the affordability check in
                            // `update_result_slot` above -- `cost_b` (the secondary
                            // ingredient) is never demand-adjusted in vanilla, only `costA`.
                            let (count_a, count_b, offer_xp) = {
                                let offer = &mut self.offers[self.selected_offer];
                                offer.uses += 1;
                                let count_a = cost_a_count(offer) as u8;
                                let count_b = offer.cost_b.as_ref().map(|c| c.0.item_count);
                                (count_a, count_b, offer.xp)
                            };

                            let input_a = self.inventory.get_stack(slot_a_index).await;
                            let mut input_a = input_a.lock().await;
                            input_a.decrement(count_a);
                            if input_a.is_empty() {
                                *input_a = ItemStack::EMPTY.clone();
                            }
                            drop(input_a);
                            self.get_behaviour().slots[slot_a_index].mark_dirty().await;

                            if let Some(count_b) = count_b {
                                let input_b = self.inventory.get_stack(slot_b_index).await;
                                let mut input_b = input_b.lock().await;
                                input_b.decrement(count_b);
                                if input_b.is_empty() {
                                    *input_b = ItemStack::EMPTY.clone();
                                }
                                drop(input_b);
                                self.get_behaviour().slots[slot_b_index].mark_dirty().await;
                            }

                            // Award XP
                            player.award_experience(offer_xp).await;

                            if let Some(on_trade) = &self.on_trade {
                                on_trade(self.selected_offer);
                            }
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

#[cfg(test)]
mod test {
    use std::borrow::Cow;
    use std::sync::Arc;

    use pumpkin_data::item::Item;
    use pumpkin_data::item_stack::ItemStack;
    use pumpkin_data::screen::WindowType;
    use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
    use pumpkin_protocol::java::client::play::MerchantOffer;
    use pumpkin_world::inventory::SimpleInventory;

    use super::{MerchantScreenHandler, SlotOrder, offer_matches, payment_slot_indices};
    use crate::screen_handler::ScreenHandlerBehaviour;

    fn offer(cost_a_count: u8, cost_b_count: Option<u8>) -> MerchantOffer {
        MerchantOffer {
            base_cost_a: ItemStackSerializer(Cow::Owned(ItemStack::new(
                cost_a_count,
                &Item::EMERALD,
            ))),
            output: ItemStackSerializer(Cow::Owned(ItemStack::new(1, &Item::DIAMOND))),
            cost_b: cost_b_count
                .map(|c| ItemStackSerializer(Cow::Owned(ItemStack::new(c, &Item::WHEAT)))),
            is_disabled: false,
            uses: 0,
            max_uses: 12,
            xp: 1,
            special_price: 0,
            price_multiplier: 0.05,
            demand: 0,
        }
    }

    fn handler(offer: MerchantOffer, slot0: ItemStack, slot1: ItemStack) -> MerchantScreenHandler {
        let mut inventory = SimpleInventory::new(3);
        inventory.stacks[0] = Arc::new(tokio::sync::Mutex::new(slot0));
        inventory.stacks[1] = Arc::new(tokio::sync::Mutex::new(slot1));
        MerchantScreenHandler {
            inventory: Arc::new(inventory),
            behaviour: ScreenHandlerBehaviour::new(0, Some(WindowType::Merchant)),
            selected_offer: 0,
            offers: vec![offer],
            on_trade: None,
        }
    }

    // Both slots holding the trade's ingredients in the "expected" order (slot 0 =
    // cost_a, slot 1 = cost_b) must match, as they always did before this fix.
    #[tokio::test]
    async fn normal_order_populates_result_slot_and_charges_correctly() {
        let slot0 = ItemStack::new(1, &Item::EMERALD);
        let slot1 = ItemStack::new(5, &Item::WHEAT);
        let order = offer_matches(&offer(1, Some(5)), &slot0, &slot1);
        assert!(matches!(order, Some(SlotOrder::Normal)));
        assert_eq!(payment_slot_indices(&order.unwrap()), (0, 1));

        let mut handler = handler(offer(1, Some(5)), slot0, slot1);
        handler.update_result_slot().await;

        let result = handler.inventory.get_stack(2).await;
        let result = result.lock().await;
        assert_eq!(result.item.id, Item::DIAMOND.id);
        assert_eq!(result.item_count, 1);
    }

    // Vanilla (`MerchantResultSlot.onTake`: `offer.take(buyA, buyB) || offer.take(buyB,
    // buyA)`) also accepts the ingredients in the swapped slots. The result slot must
    // still populate, and the slot->ingredient mapping used to consume inputs must be
    // swapped too.
    #[tokio::test]
    async fn swapped_order_populates_result_slot_and_charges_correctly() {
        let slot0 = ItemStack::new(5, &Item::WHEAT);
        let slot1 = ItemStack::new(1, &Item::EMERALD);
        let order = offer_matches(&offer(1, Some(5)), &slot0, &slot1);
        assert!(matches!(order, Some(SlotOrder::Swapped)));
        // Swapped: cost_a (emerald) lives in slot 1, cost_b (wheat) lives in slot 0.
        assert_eq!(payment_slot_indices(&order.unwrap()), (1, 0));

        let mut handler = handler(offer(1, Some(5)), slot0, slot1);
        handler.update_result_slot().await;

        let result = handler.inventory.get_stack(2).await;
        let result = result.lock().await;
        assert_eq!(result.item.id, Item::DIAMOND.id);
        assert_eq!(result.item_count, 1);
    }

    #[test]
    fn mismatched_ingredients_do_not_match_either_order() {
        let offer = offer(1, Some(5));
        let slot0 = ItemStack::new(1, &Item::WHEAT);
        let slot1 = ItemStack::new(5, &Item::EMERALD);

        assert!(offer_matches(&offer, &slot0, &slot1).is_none());
    }

    #[test]
    fn insufficient_count_in_either_order_does_not_match() {
        let offer = offer(4, Some(5));
        // Not enough emeralds in either slot to cover cost_a's count of 4.
        let slot0 = ItemStack::new(1, &Item::EMERALD);
        let slot1 = ItemStack::new(5, &Item::WHEAT);

        assert!(offer_matches(&offer, &slot0, &slot1).is_none());
        assert!(offer_matches(&offer, &slot1, &slot0).is_none());
    }
}
