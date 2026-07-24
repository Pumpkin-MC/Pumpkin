use std::any::Any;
use std::sync::{Arc, Mutex};

use pumpkin_data::{item_stack::ItemStack, screen::WindowType};
use pumpkin_protocol::java::client::play::MerchantOffer;
use pumpkin_world::inventory::Inventory;

use crate::{
    player::player_inventory::PlayerInventory,
    screen_handler::{
        InventoryPlayer, ItemStackFuture, ScreenHandler, ScreenHandlerBehaviour,
        ScreenHandlerFuture, offer_or_drop_stack,
    },
    slot::{BoxFuture, NormalSlot, TakeOnlySlot, TakeSlotCharge},
};

pub struct MerchantScreenHandler {
    pub inventory: Arc<dyn Inventory>,
    behaviour: ScreenHandlerBehaviour,
    /// Trade state shared with the output slot's charge hook.
    trade: Arc<Mutex<MerchantTrade>>,
    pub on_trade: Option<Box<dyn Fn(usize) + Send + Sync>>,
}

/// Trade state shared between the screen handler and its output slot.
struct MerchantTrade {
    selected_offer: usize,
    offers: Vec<MerchantOffer>,
    /// Offers charged by the take hook, not yet forwarded to `on_trade`.
    charged: Vec<usize>,
}

/// Charges one trade on the merchant output slot.
///
/// Fires from the slot's take path, so every verified take — pickup,
/// shift-click, throw, swap — consumes the offer's inputs and counts
/// exactly one use; there is no per-action gate that could forget one.
struct MerchantSlotCharge {
    inventory: Arc<dyn Inventory>,
    trade: Arc<Mutex<MerchantTrade>>,
}

impl TakeSlotCharge for MerchantSlotCharge {
    fn can_take(&self, _player: &dyn InventoryPlayer) -> bool {
        let trade = self.trade.lock().unwrap();
        trade
            .offers
            .get(trade.selected_offer)
            .is_some_and(|offer| !offer.is_disabled && offer.uses < offer.max_uses)
    }

    fn on_take<'a>(
        &'a self,
        player: &'a dyn InventoryPlayer,
        _stack: &'a ItemStack,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            let (count_a, count_b, offer_xp) = {
                let mut trade = self.trade.lock().unwrap();
                let selected = trade.selected_offer;
                let Some(offer) = trade.offers.get_mut(selected) else {
                    return;
                };
                if offer.is_disabled || offer.uses >= offer.max_uses {
                    return;
                }
                offer.uses += 1;
                let data = (
                    offer.base_cost_a.0.item_count,
                    offer.cost_b.as_ref().map(|c| c.0.item_count),
                    offer.xp,
                );
                trade.charged.push(selected);
                data
            };

            let input_a = self.inventory.get_stack(0).await;
            let mut input_a = input_a.lock().await;
            input_a.decrement(count_a);
            if input_a.is_empty() {
                *input_a = ItemStack::EMPTY.clone();
            }
            drop(input_a);

            if let Some(count_b) = count_b {
                let input_b = self.inventory.get_stack(1).await;
                let mut input_b = input_b.lock().await;
                input_b.decrement(count_b);
                if input_b.is_empty() {
                    *input_b = ItemStack::EMPTY.clone();
                }
                drop(input_b);
            }
            self.inventory.mark_dirty();

            player.award_experience(offer_xp).await;
        })
    }
}

impl MerchantScreenHandler {
    pub async fn new(
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        inventory: Arc<dyn Inventory>,
        offers: Vec<MerchantOffer>,
    ) -> Self {
        let trade = Arc::new(Mutex::new(MerchantTrade {
            selected_offer: 0,
            offers,
            charged: Vec::new(),
        }));
        let mut handler = Self {
            inventory: inventory.clone(),
            behaviour: ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Merchant)),
            trade: trade.clone(),
            on_trade: None,
        };
        inventory.on_open().await;

        // Merchant: 2 input slots + take-only output (PickupAll must not sweep output).
        // The output slot owns the charge hook so every take path charges once.
        handler.add_slot(Arc::new(NormalSlot::new(inventory.clone(), 0)));
        handler.add_slot(Arc::new(NormalSlot::new(inventory.clone(), 1)));
        handler.add_slot(Arc::new(TakeOnlySlot::with_charge(
            inventory.clone(),
            2,
            Arc::new(MerchantSlotCharge {
                inventory: inventory.clone(),
                trade,
            }),
        )));
        handler.behaviour.container_slots = handler.behaviour.slots.len();

        let player_inventory: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&player_inventory);

        handler
    }

    pub async fn set_selected_offer(&mut self, index: usize) {
        self.trade.lock().unwrap().selected_offer = index;
        self.update_result_slot().await;
        self.send_content_updates().await;
    }

    pub async fn update_result_slot(&mut self) {
        let offer_data = {
            let trade = self.trade.lock().unwrap();
            trade.offers.get(trade.selected_offer).map(|offer| {
                (
                    (*offer.base_cost_a.0).clone(),
                    offer.cost_b.as_ref().map(|c| (*c.0).clone()),
                    (*offer.output.0).clone(),
                )
            })
        };

        let Some((cost_a, cost_b, output)) = offer_data else {
            self.inventory.set_stack(2, ItemStack::EMPTY.clone()).await;
            return;
        };

        let input_a = self.inventory.get_stack(0).await;
        let input_a = input_a.lock().await;
        let input_b = self.inventory.get_stack(1).await;
        let input_b = input_b.lock().await;

        let match_a = input_a.are_items_and_components_equal(&cost_a)
            && input_a.item_count >= cost_a.item_count;

        let match_b = cost_b.map_or_else(
            || input_b.is_empty(),
            |cost_b| {
                input_b.are_items_and_components_equal(&cost_b)
                    && input_b.item_count >= cost_b.item_count
            },
        );

        if match_a && match_b {
            self.inventory.set_stack(2, output).await;
        } else {
            self.inventory.set_stack(2, ItemStack::EMPTY.clone()).await;
        }
    }

    /// Forwards trades charged by the output slot's take hook to `on_trade`.
    fn forward_charged_trades(&self) {
        let charged = {
            let mut trade = self.trade.lock().unwrap();
            std::mem::take(&mut trade.charged)
        };
        if let Some(on_trade) = &self.on_trade {
            for offer_index in charged {
                on_trade(offer_index);
            }
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
            // Forward any trades taken through non-click paths (e.g. direct
            // slot takes) before the screen state is torn down.
            self.forward_charged_trades();
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
        player: &'a dyn InventoryPlayer,
        slot_index: i32,
    ) -> ItemStackFuture<'a> {
        Box::pin(async move {
            let slot = self.get_behaviour().slots[slot_index as usize].clone();

            if slot.has_stack().await {
                let slot_stack_lock = slot.get_stack().await;
                let mut slot_stack_mut = slot_stack_lock.lock().await;
                let stack_prev = slot_stack_mut.clone();

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

                let stack = slot_stack_mut.clone();
                drop(slot_stack_mut);
                if stack.is_empty() {
                    slot.set_stack(ItemStack::EMPTY.clone()).await;
                } else {
                    slot.mark_dirty().await;
                }
                if stack.item_count == stack_prev.item_count {
                    return ItemStack::EMPTY.clone();
                }

                // Charge only what was actually moved; a failed delivery above
                // returns before this point and charges nothing.
                let mut taken_stack = stack_prev.clone();
                taken_stack.set_count(stack_prev.item_count - stack.item_count);
                slot.on_take_item(player, &taken_stack).await;

                return stack_prev;
            }

            ItemStack::EMPTY.clone()
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
            self.internal_on_slot_click(slot_index, button, action_type, player)
                .await;
            self.forward_charged_trades();
            if slot_index == 0 || slot_index == 1 || slot_index == 2 {
                self.update_result_slot().await;
                self.send_content_updates().await;
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use pumpkin_data::item::Item;
    use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
    use pumpkin_protocol::java::server::play::SlotActionType;
    use pumpkin_world::inventory::SimpleInventory;

    use crate::screen_handler::ScreenHandler;
    use crate::test_util::MockPlayer;

    fn test_offer() -> MerchantOffer {
        MerchantOffer {
            base_cost_a: ItemStackSerializer(Cow::Owned(ItemStack::new(5, &Item::EMERALD))),
            output: ItemStackSerializer(Cow::Owned(ItemStack::new(2, &Item::DIAMOND))),
            cost_b: Some(ItemStackSerializer(Cow::Owned(ItemStack::new(
                3,
                &Item::COAL,
            )))),
            is_disabled: false,
            uses: 0,
            max_uses: 12,
            xp: 7,
            special_price: 0,
            price_multiplier: 0.0,
            demand: 0,
        }
    }

    async fn setup() -> (MerchantScreenHandler, Arc<MockPlayer>) {
        let player = Arc::new(MockPlayer::new());
        let inventory: Arc<dyn Inventory> = Arc::new(SimpleInventory::new(3));
        inventory
            .set_stack(0, ItemStack::new(10, &Item::EMERALD))
            .await;
        inventory.set_stack(1, ItemStack::new(6, &Item::COAL)).await;
        let mut handler =
            MerchantScreenHandler::new(1, &player.player_inventory, inventory, vec![test_offer()])
                .await;
        handler.update_result_slot().await;
        assert_eq!(handler.get_behaviour().container_slots, 3);
        (handler, player)
    }

    async fn inv_stack(inventory: &Arc<dyn Inventory>, index: usize) -> ItemStack {
        inventory.get_stack(index).await.lock().await.clone()
    }

    fn uses(handler: &MerchantScreenHandler) -> i32 {
        handler.trade.lock().unwrap().offers[0].uses
    }

    fn xp_awarded(player: &MockPlayer) -> i32 {
        player.xp_awarded.load(Ordering::Relaxed)
    }

    async fn dropped_count(player: &MockPlayer) -> usize {
        player.dropped.lock().await.len()
    }

    fn assert_charged_once(handler: &MerchantScreenHandler, player: &MockPlayer) {
        assert_eq!(uses(handler), 1, "exactly one trade use must be counted");
        assert_eq!(xp_awarded(player), 7, "trade XP must be awarded once");
    }

    #[tokio::test]
    async fn throw_on_output_charges_exactly_once() {
        let (mut handler, player) = setup().await;
        assert_eq!(inv_stack(&handler.inventory, 2).await.item_count, 2);

        // Q on the result slot: takes one item, pays the full trade once.
        handler
            .on_slot_click(2, 0, SlotActionType::Throw, player.as_ref())
            .await;
        assert_charged_once(&handler, &player);
        assert_eq!(dropped_count(&player).await, 1);
        assert_eq!(inv_stack(&handler.inventory, 0).await.item_count, 5);
        assert_eq!(inv_stack(&handler.inventory, 1).await.item_count, 3);
        // Inputs still cover a trade, so the result refills.
        assert_eq!(inv_stack(&handler.inventory, 2).await.item_count, 2);

        // Second take consumes the remaining inputs; the cycle then dies.
        handler
            .on_slot_click(2, 0, SlotActionType::Throw, player.as_ref())
            .await;
        assert_eq!(uses(&handler), 2);
        assert_eq!(dropped_count(&player).await, 2);
        assert!(inv_stack(&handler.inventory, 0).await.is_empty());
        assert!(inv_stack(&handler.inventory, 1).await.is_empty());
        assert!(inv_stack(&handler.inventory, 2).await.is_empty());

        // No inputs -> no result -> further throws take and charge nothing.
        handler
            .on_slot_click(2, 0, SlotActionType::Throw, player.as_ref())
            .await;
        assert_eq!(uses(&handler), 2);
        assert_eq!(dropped_count(&player).await, 2);
        assert_eq!(xp_awarded(&player), 14);
    }

    #[tokio::test]
    async fn throw_all_on_output_charges_once() {
        let (mut handler, player) = setup().await;

        // Ctrl-Q on the result slot: the whole stack drops, charged once.
        handler
            .on_slot_click(2, 1, SlotActionType::Throw, player.as_ref())
            .await;
        assert_charged_once(&handler, &player);
        let dropped = player.dropped.lock().await;
        assert_eq!(dropped.len(), 1);
        assert_eq!(dropped[0].item_count, 2);
        drop(dropped);
        assert_eq!(inv_stack(&handler.inventory, 0).await.item_count, 5);
        assert_eq!(inv_stack(&handler.inventory, 1).await.item_count, 3);
    }

    #[tokio::test]
    async fn swap_on_output_charges_once() {
        let (mut handler, player) = setup().await;

        // Number-key swap onto an empty hotbar slot takes the result.
        handler
            .on_slot_click(2, 0, SlotActionType::Swap, player.as_ref())
            .await;
        assert_charged_once(&handler, &player);
        let hotbar = player.player_inventory.get_stack(0).await;
        let hotbar = hotbar.lock().await;
        assert_eq!(hotbar.item.id, Item::DIAMOND.id);
        assert_eq!(hotbar.item_count, 2);
        drop(hotbar);
        assert_eq!(inv_stack(&handler.inventory, 0).await.item_count, 5);
        assert_eq!(inv_stack(&handler.inventory, 1).await.item_count, 3);
        assert_eq!(inv_stack(&handler.inventory, 2).await.item_count, 2);
    }

    #[tokio::test]
    async fn pickup_all_on_output_is_noop() {
        let (mut handler, player) = setup().await;
        *handler.get_behaviour_mut().cursor_stack.lock().await = ItemStack::new(1, &Item::DIAMOND);

        // Double-click sweep must skip the take-only output slot entirely.
        handler
            .on_slot_click(2, 0, SlotActionType::PickupAll, player.as_ref())
            .await;
        assert_eq!(uses(&handler), 0);
        assert_eq!(xp_awarded(&player), 0);
        assert_eq!(inv_stack(&handler.inventory, 2).await.item_count, 2);
        let cursor = handler.get_behaviour().cursor_stack.lock().await;
        assert_eq!(cursor.item_count, 1);
    }

    #[tokio::test]
    async fn quick_move_into_full_inventory_charges_nothing() {
        let (mut handler, player) = setup().await;
        for i in 0..36 {
            player
                .player_inventory
                .set_stack(i, ItemStack::new(64, &Item::COBBLESTONE))
                .await;
        }

        handler
            .on_slot_click(2, 0, SlotActionType::QuickMove, player.as_ref())
            .await;
        assert_eq!(uses(&handler), 0, "failed delivery must not charge");
        assert_eq!(xp_awarded(&player), 0);
        assert_eq!(inv_stack(&handler.inventory, 2).await.item_count, 2);
        assert_eq!(inv_stack(&handler.inventory, 0).await.item_count, 10);
        assert_eq!(inv_stack(&handler.inventory, 1).await.item_count, 6);
    }

    #[tokio::test]
    async fn quick_move_on_output_charges_once() {
        let (mut handler, player) = setup().await;

        handler
            .on_slot_click(2, 0, SlotActionType::QuickMove, player.as_ref())
            .await;
        assert_charged_once(&handler, &player);
        let mut moved = 0;
        for i in 0..36 {
            let stack = player.player_inventory.get_stack(i).await;
            let stack = stack.lock().await;
            if stack.item.id == Item::DIAMOND.id {
                moved += stack.item_count;
            }
        }
        assert_eq!(moved, 2, "the result stack must reach the player");
        assert_eq!(inv_stack(&handler.inventory, 2).await.item_count, 2);
    }

    #[tokio::test]
    async fn pickup_with_incompatible_cursor_charges_nothing() {
        let (mut handler, player) = setup().await;
        *handler.get_behaviour_mut().cursor_stack.lock().await = ItemStack::new(1, &Item::DIRT);

        handler
            .on_slot_click(2, 0, SlotActionType::Pickup, player.as_ref())
            .await;
        assert_eq!(uses(&handler), 0, "no delivery happened, so no charge");
        assert_eq!(xp_awarded(&player), 0);
        assert_eq!(inv_stack(&handler.inventory, 2).await.item_count, 2);
        let cursor = handler.get_behaviour().cursor_stack.lock().await;
        assert_eq!(cursor.item.id, Item::DIRT.id);
    }

    #[tokio::test]
    async fn pickup_on_output_charges_once() {
        let (mut handler, player) = setup().await;

        handler
            .on_slot_click(2, 0, SlotActionType::Pickup, player.as_ref())
            .await;
        assert_charged_once(&handler, &player);
        let cursor = handler.get_behaviour().cursor_stack.lock().await;
        assert_eq!(cursor.item.id, Item::DIAMOND.id);
        assert_eq!(cursor.item_count, 2);
        drop(cursor);
        assert_eq!(inv_stack(&handler.inventory, 0).await.item_count, 5);
        assert_eq!(inv_stack(&handler.inventory, 1).await.item_count, 3);
        assert_eq!(inv_stack(&handler.inventory, 2).await.item_count, 2);
    }

    #[tokio::test]
    async fn depleted_offer_blocks_every_take_path() {
        let (mut handler, player) = setup().await;
        handler.trade.lock().unwrap().offers[0].uses = 12;

        for action in [SlotActionType::Pickup, SlotActionType::Throw] {
            handler.on_slot_click(2, 0, action, player.as_ref()).await;
        }
        handler
            .on_slot_click(2, 0, SlotActionType::Swap, player.as_ref())
            .await;
        handler
            .on_slot_click(2, 0, SlotActionType::QuickMove, player.as_ref())
            .await;

        assert_eq!(uses(&handler), 12);
        assert_eq!(xp_awarded(&player), 0);
        assert_eq!(dropped_count(&player).await, 0);
        assert_eq!(inv_stack(&handler.inventory, 0).await.item_count, 10);
        assert!(
            player
                .player_inventory
                .get_stack(0)
                .await
                .lock()
                .await
                .is_empty()
        );
    }

    #[tokio::test]
    async fn on_trade_callback_fires_exactly_once_per_take() {
        let (mut handler, player) = setup().await;
        let trade_count = Arc::new(AtomicUsize::new(0));
        let counter = trade_count.clone();
        handler.on_trade = Some(Box::new(move |_offer_index| {
            counter.fetch_add(1, Ordering::Relaxed);
        }));

        handler
            .on_slot_click(2, 0, SlotActionType::Throw, player.as_ref())
            .await;
        assert_eq!(trade_count.load(Ordering::Relaxed), 1);

        handler
            .on_slot_click(2, 0, SlotActionType::Pickup, player.as_ref())
            .await;
        assert_eq!(trade_count.load(Ordering::Relaxed), 2);
    }
}
