use std::{
    any::Any,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU8, Ordering},
    },
};

use pumpkin_data::{
    item_stack::ItemStack,
    screen::WindowType,
    statistic::{CustomStatistic, StatisticCategory},
};
use pumpkin_world::inventory::Inventory;

use crate::{
    player::player_inventory::PlayerInventory,
    screen_handler::{
        InventoryPlayer, ItemStackFuture, ScreenHandler, ScreenHandlerBehaviour,
        ScreenHandlerFuture, offer_or_drop_stack,
    },
    slot::{BoxFuture, NormalSlot, Slot},
};

type MerchantValidityCheck = Box<dyn Fn(&dyn InventoryPlayer) -> bool + Send + Sync>;

pub struct MerchantScreenHandler {
    pub inventory: Arc<dyn Inventory>,
    behaviour: ScreenHandlerBehaviour,
    selected_offer: usize,
    active_offer: Option<usize>,
    pub offers: Vec<pumpkin_protocol::java::client::play::MerchantOffer>,
    pub on_trade: Option<Box<dyn Fn(usize) -> ScreenHandlerFuture<'static, ()> + Send + Sync>>,
    pub on_trade_updated: Option<Box<dyn Fn(bool) + Send + Sync>>,
    pub on_close: Option<Box<dyn Fn() -> ScreenHandlerFuture<'static, ()> + Send + Sync>>,
    pub validity_check: Option<MerchantValidityCheck>,
    result_taken: Arc<AtomicBool>,
}

impl MerchantScreenHandler {
    pub async fn new(
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        inventory: Arc<dyn Inventory>,
        offers: Vec<pumpkin_protocol::java::client::play::MerchantOffer>,
    ) -> Self {
        let result_taken = Arc::new(AtomicBool::new(false));
        let mut behaviour = ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Merchant));
        behaviour.container_slots = 3;
        let mut handler = Self {
            inventory: inventory.clone(),
            behaviour,
            selected_offer: 0,
            active_offer: None,
            offers,
            on_trade: None,
            on_trade_updated: None,
            on_close: None,
            validity_check: None,
            result_taken: result_taken.clone(),
        };

        inventory.on_open().await;

        for i in 0..2 {
            handler.add_slot(Arc::new(NormalSlot::new(inventory.clone(), i)));
        }
        handler.add_slot(Arc::new(MerchantResultSlot::new(
            inventory.clone(),
            2,
            result_taken,
        )));

        let player_inventory: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&player_inventory);

        handler
    }

    pub async fn set_selected_offer(&mut self, index: usize) {
        self.selected_offer = index;
        self.try_move_items(index).await;
        self.update_result_slot().await;
        self.notify_trade_updated().await;
        self.send_content_updates().await;
    }

    async fn try_move_items(&mut self, index: usize) {
        let Some(offer) = self.offers.get(index) else {
            return;
        };
        let cost_a = offer.base_cost_a.0.as_ref().clone();
        let cost_b = offer.cost_b.as_ref().map(|cost| cost.0.as_ref().clone());
        let player_slots_end = self.get_behaviour().slots.len() as i32;

        for index in 0..2 {
            let stack = self.inventory.get_stack(index).await;
            let mut stack = stack.lock().await;
            if !stack.is_empty()
                && !self
                    .insert_item(&mut stack, 3, player_slots_end, true)
                    .await
            {
                return;
            }
            if stack.is_empty() {
                *stack = ItemStack::EMPTY.clone();
            }
        }

        if !self.inventory.get_stack(0).await.lock().await.is_empty()
            || !self.inventory.get_stack(1).await.lock().await.is_empty()
        {
            return;
        }

        self.move_from_inventory_to_payment_slot(0, &cost_a).await;
        if let Some(cost_b) = cost_b {
            self.move_from_inventory_to_payment_slot(1, &cost_b).await;
        }
    }

    fn adjusted_cost_a(offer: &pumpkin_protocol::java::client::play::MerchantOffer) -> ItemStack {
        let mut cost = offer.base_cost_a.0.as_ref().clone();
        let demand = i32::from(cost.item_count).saturating_mul(offer.demand) as f32;
        let demand_bonus = (demand * offer.price_multiplier).floor().max(0.0) as i32;
        let count = i32::from(cost.item_count)
            .saturating_add(demand_bonus)
            .saturating_add(offer.special_price)
            .clamp(1, i32::from(cost.get_max_stack_size()));
        cost.set_count(count as u8);
        cost
    }

    fn matches_cost(cost: &ItemStack, input: &ItemStack) -> bool {
        !input.is_empty()
            && cost.are_items_and_components_equal(input)
            && input.item_count >= cost.item_count
    }

    fn satisfies(
        offer: &pumpkin_protocol::java::client::play::MerchantOffer,
        input_a: &ItemStack,
        input_b: &ItemStack,
    ) -> bool {
        Self::matches_cost(&Self::adjusted_cost_a(offer), input_a)
            && offer.cost_b.as_ref().map_or_else(
                || input_b.is_empty(),
                |cost_b| Self::matches_cost(&cost_b.0, input_b),
            )
    }

    fn input_order(
        offer: &pumpkin_protocol::java::client::play::MerchantOffer,
        input_a: &ItemStack,
        input_b: &ItemStack,
    ) -> Option<bool> {
        if Self::satisfies(offer, input_a, input_b) {
            Some(false)
        } else if Self::satisfies(offer, input_b, input_a) {
            Some(true)
        } else {
            None
        }
    }

    fn matching_offer(&self, input_a: &ItemStack, input_b: &ItemStack) -> Option<usize> {
        if self.selected_offer > 0 && self.selected_offer < self.offers.len() {
            return Self::satisfies(&self.offers[self.selected_offer], input_a, input_b)
                .then_some(self.selected_offer);
        }

        self.offers
            .iter()
            .position(|offer| Self::satisfies(offer, input_a, input_b))
    }

    fn find_active_offer(&self, input_a: &ItemStack, input_b: &ItemStack) -> Option<usize> {
        self.matching_offer(input_a, input_b)
            .filter(|&index| !self.offers[index].is_out_of_stock())
            .or_else(|| {
                self.matching_offer(input_b, input_a)
                    .filter(|&index| !self.offers[index].is_out_of_stock())
            })
    }

    async fn consume_payment(&self, slot: usize, count: u8) {
        let stack = self.inventory.get_stack(slot).await;
        let mut stack = stack.lock().await;
        stack.decrement(count);
        if stack.is_empty() {
            *stack = ItemStack::EMPTY.clone();
        }
        drop(stack);
        self.get_behaviour().slots[slot].mark_dirty().await;
    }

    async fn complete_trade(&mut self, player: &dyn InventoryPlayer) -> bool {
        let Some(offer_index) = self.active_offer else {
            return false;
        };
        let Some(offer) = self.offers.get(offer_index) else {
            return false;
        };
        if offer.is_out_of_stock() {
            return false;
        }

        let input_a = self.inventory.get_stack(0).await.lock().await.clone();
        let input_b = self.inventory.get_stack(1).await.lock().await.clone();
        let Some(swapped) = Self::input_order(offer, &input_a, &input_b) else {
            return false;
        };
        let cost_a = Self::adjusted_cost_a(offer).item_count;
        let cost_b = offer.cost_b.as_ref().map(|cost| cost.0.item_count);

        self.consume_payment(usize::from(swapped), cost_a).await;
        if let Some(cost_b) = cost_b {
            self.consume_payment(usize::from(!swapped), cost_b).await;
        }
        self.offers[offer_index].uses += 1;

        if let Some(on_trade) = &self.on_trade {
            on_trade(offer_index).await;
        }
        player
            .increment_stat(
                StatisticCategory::Custom,
                CustomStatistic::TradedWithVillager as i32,
                1,
            )
            .await;
        true
    }

    async fn can_fully_insert(&self, stack: &ItemStack, start: usize, end: usize) -> bool {
        let mut remaining = stack.item_count;
        for slot in &self.get_behaviour().slots[start..end] {
            if !slot.can_insert(stack).await {
                continue;
            }
            let existing = slot.get_cloned_stack().await;
            let capacity = if existing.is_empty() {
                slot.get_max_item_count_for_stack(stack).await
            } else if existing.are_items_and_components_equal(stack)
                && stack.are_items_and_components_equal(&existing)
            {
                slot.get_max_item_count_for_stack(&existing)
                    .await
                    .saturating_sub(existing.item_count)
            } else {
                0
            };
            remaining = remaining.saturating_sub(capacity);
            if remaining == 0 {
                return true;
            }
        }
        false
    }

    async fn move_from_inventory_to_payment_slot(&self, payment_slot: usize, cost: &ItemStack) {
        let payment = self.inventory.get_stack(payment_slot).await;
        let mut payment = payment.lock().await;

        for slot in &self.get_behaviour().slots[3..] {
            let source = slot.get_stack().await;
            let mut source = source.lock().await;
            if source.is_empty()
                || source.item.id != cost.item.id
                || !cost.are_items_and_components_equal(&source)
                || (!payment.is_empty()
                    && (!source.are_items_and_components_equal(&payment)
                        || !payment.are_items_and_components_equal(&source)))
            {
                continue;
            }

            if payment.is_empty() {
                *payment = source.copy_with_count(0);
            }
            let moved = source
                .item_count
                .min(source.get_max_stack_size() - payment.item_count);
            payment.increment(moved);
            source.decrement(moved);
            if source.is_empty() {
                *source = ItemStack::EMPTY.clone();
            }
            if payment.item_count == payment.get_max_stack_size() {
                break;
            }
        }
    }

    pub async fn update_result_slot(&mut self) {
        let input_a = self.inventory.get_stack(0).await.lock().await.clone();
        let input_b = self.inventory.get_stack(1).await.lock().await.clone();
        self.active_offer = self.find_active_offer(&input_a, &input_b);

        if let Some(index) = self.active_offer {
            self.inventory
                .set_stack(2, (*self.offers[index].output.0).clone())
                .await;
        } else {
            self.inventory.set_stack(2, ItemStack::EMPTY.clone()).await;
        }
    }

    async fn notify_trade_updated(&self) {
        if self.inventory.get_stack(0).await.lock().await.is_empty()
            && self.inventory.get_stack(1).await.lock().await.is_empty()
        {
            return;
        }
        if let Some(on_trade_updated) = &self.on_trade_updated {
            on_trade_updated(self.active_offer.is_some());
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

    fn can_use(&self, player: &dyn InventoryPlayer) -> bool {
        self.validity_check
            .as_ref()
            .is_none_or(|check| check(player))
    }

    fn get_behaviour(&self) -> &ScreenHandlerBehaviour {
        &self.behaviour
    }

    fn get_behaviour_mut(&mut self) -> &mut ScreenHandlerBehaviour {
        &mut self.behaviour
    }

    fn on_closed<'a>(&'a mut self, player: &'a dyn InventoryPlayer) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            if let Some(on_close) = &self.on_close {
                on_close().await;
            }
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
            const PLAYER_INVENTORY_START: i32 = 3;
            const PLAYER_HOTBAR_START: i32 = 30;

            let slot = self.get_behaviour().slots[slot_index as usize].clone();
            let mut stack = slot.get_cloned_stack().await;
            if stack.is_empty() {
                return ItemStack::EMPTY.clone();
            }
            let original = stack.clone();
            let player_slots_end = self.get_behaviour().slots.len() as i32;

            if slot_index == 2 {
                if !self
                    .can_fully_insert(
                        &stack,
                        PLAYER_INVENTORY_START as usize,
                        player_slots_end as usize,
                    )
                    .await
                    || !self
                        .insert_item(&mut stack, PLAYER_INVENTORY_START, player_slots_end, true)
                        .await
                {
                    return ItemStack::EMPTY.clone();
                }
            } else if slot_index < PLAYER_INVENTORY_START {
                if !self
                    .insert_item(&mut stack, PLAYER_INVENTORY_START, player_slots_end, false)
                    .await
                {
                    return ItemStack::EMPTY.clone();
                }
            } else if slot_index < PLAYER_HOTBAR_START {
                if !self
                    .insert_item(&mut stack, PLAYER_HOTBAR_START, player_slots_end, false)
                    .await
                {
                    return ItemStack::EMPTY.clone();
                }
            } else if !self
                .insert_item(
                    &mut stack,
                    PLAYER_INVENTORY_START,
                    PLAYER_HOTBAR_START,
                    false,
                )
                .await
            {
                return ItemStack::EMPTY.clone();
            }

            if stack.item_count == original.item_count {
                return ItemStack::EMPTY.clone();
            }
            if stack.is_empty() {
                slot.set_stack(ItemStack::EMPTY.clone()).await;
            } else {
                slot.set_stack(stack.clone()).await;
            }

            let mut taken = original.clone();
            taken.set_count(original.item_count - stack.item_count);
            slot.on_take_item(player, &taken).await;
            if slot_index == 2 {
                slot.on_quick_move_crafted(stack, original.clone()).await;
                self.result_taken.store(false, Ordering::Relaxed);
                if !self.complete_trade(player).await {
                    return ItemStack::EMPTY.clone();
                }
                self.update_result_slot().await;
            }

            original
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
                self.update_result_slot().await;
            }
            self.result_taken.store(false, Ordering::Relaxed);
            self.internal_on_slot_click(slot_index, button, action_type, player)
                .await;
            if self.result_taken.swap(false, Ordering::Relaxed) {
                self.complete_trade(player).await;
            }
            self.update_result_slot().await;
            self.notify_trade_updated().await;
            self.send_content_updates().await;
        })
    }
}

struct MerchantResultSlot {
    inventory: Arc<dyn Inventory>,
    index: usize,
    id: AtomicU8,
    result_taken: Arc<AtomicBool>,
}

impl MerchantResultSlot {
    fn new(inventory: Arc<dyn Inventory>, index: usize, result_taken: Arc<AtomicBool>) -> Self {
        Self {
            inventory,
            index,
            id: AtomicU8::new(0),
            result_taken,
        }
    }
}

impl Slot for MerchantResultSlot {
    fn get_inventory(&self) -> Arc<dyn Inventory> {
        self.inventory.clone()
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_id(&self, id: usize) {
        self.id.store(id as u8, Ordering::Relaxed);
    }

    fn take_stack(&self, _amount: u8) -> BoxFuture<'_, ItemStack> {
        Box::pin(async move { self.inventory.remove_stack(self.index).await })
    }

    fn on_take_item<'a>(
        &'a self,
        _player: &'a dyn InventoryPlayer,
        _stack: &'a ItemStack,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            self.result_taken.store(true, Ordering::Relaxed);
            self.mark_dirty().await;
        })
    }

    fn can_insert(&self, _stack: &ItemStack) -> BoxFuture<'_, bool> {
        Box::pin(async { false })
    }

    fn mark_dirty(&self) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.inventory.mark_dirty();
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{
        borrow::Cow,
        collections::HashMap,
        sync::atomic::{AtomicI32, AtomicUsize, Ordering},
    };

    use pumpkin_data::{data_component_impl::EquipmentSlot, item::Item};
    use pumpkin_protocol::{
        codec::item_stack_seralizer::ItemStackSerializer,
        java::{
            client::play::{
                CSetContainerContent, CSetContainerProperty, CSetContainerSlot, CSetCursorItem,
                CSetPlayerInventory, CSetSelectedSlot,
            },
            server::play::SlotActionType,
        },
    };
    use pumpkin_world::inventory::SimpleInventory;
    use tokio::sync::Mutex;

    use crate::{
        entity_equipment::EntityEquipment,
        screen_handler::{InventoryPlayer, PlayerFuture},
    };

    use super::*;

    fn single_cost_offer(
        cost: &'static Item,
        cost_count: u8,
        output: &'static Item,
        max_uses: i32,
    ) -> pumpkin_protocol::java::client::play::MerchantOffer {
        pumpkin_protocol::java::client::play::MerchantOffer {
            base_cost_a: ItemStackSerializer(Cow::Owned(ItemStack::new(cost_count, cost))),
            output: ItemStackSerializer(Cow::Owned(ItemStack::new(1, output))),
            cost_b: None,
            reward_exp: true,
            uses: 0,
            max_uses,
            xp: 2,
            special_price: 0,
            price_multiplier: 0.05,
            demand: 0,
        }
    }

    fn bookshelf_offer() -> pumpkin_protocol::java::client::play::MerchantOffer {
        single_cost_offer(&Item::EMERALD, 9, &Item::BOOKSHELF, 12)
    }

    struct TestPlayer {
        inventory: Arc<PlayerInventory>,
        experience: AtomicI32,
        traded: AtomicI32,
    }

    impl TestPlayer {
        fn new(inventory: Arc<PlayerInventory>) -> Self {
            Self {
                inventory,
                experience: AtomicI32::new(0),
                traded: AtomicI32::new(0),
            }
        }
    }

    impl InventoryPlayer for TestPlayer {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn drop_item(&self, _item: ItemStack, _retain_ownership: bool) -> PlayerFuture<'_, ()> {
            Box::pin(async {})
        }

        fn get_inventory(&self) -> Arc<PlayerInventory> {
            self.inventory.clone()
        }

        fn has_infinite_materials(&self) -> bool {
            false
        }

        fn is_creative(&self) -> bool {
            false
        }

        fn experience_level(&self) -> i32 {
            0
        }

        fn add_experience_levels(&self, _levels: i32) -> PlayerFuture<'_, ()> {
            Box::pin(async {})
        }

        fn enchantment_seed(&self) -> i32 {
            0
        }

        fn set_enchantment_seed(&self, _seed: i32) -> PlayerFuture<'_, ()> {
            Box::pin(async {})
        }

        fn enqueue_inventory_packet<'a>(
            &'a self,
            _packet: &'a CSetContainerContent,
        ) -> PlayerFuture<'a, ()> {
            Box::pin(async {})
        }

        fn enqueue_slot_packet<'a>(
            &'a self,
            _packet: &'a CSetContainerSlot,
        ) -> PlayerFuture<'a, ()> {
            Box::pin(async {})
        }

        fn enqueue_cursor_packet<'a>(
            &'a self,
            _packet: &'a CSetCursorItem,
        ) -> PlayerFuture<'a, ()> {
            Box::pin(async {})
        }

        fn enqueue_property_packet<'a>(
            &'a self,
            _packet: &'a CSetContainerProperty,
        ) -> PlayerFuture<'a, ()> {
            Box::pin(async {})
        }

        fn enqueue_slot_set_packet<'a>(
            &'a self,
            _packet: &'a CSetPlayerInventory,
        ) -> PlayerFuture<'a, ()> {
            Box::pin(async {})
        }

        fn enqueue_set_held_item_packet<'a>(
            &'a self,
            _packet: &'a CSetSelectedSlot,
        ) -> PlayerFuture<'a, ()> {
            Box::pin(async {})
        }

        fn enqueue_equipment_change<'a>(
            &'a self,
            _slot: &'a EquipmentSlot,
            _stack: &'a ItemStack,
        ) -> PlayerFuture<'a, ()> {
            Box::pin(async {})
        }

        fn award_experience(&self, amount: i32) -> PlayerFuture<'_, ()> {
            Box::pin(async move {
                self.experience.fetch_add(amount, Ordering::Relaxed);
            })
        }

        fn increment_stat(
            &self,
            category: StatisticCategory,
            stat_id: i32,
            amount: i32,
        ) -> PlayerFuture<'_, ()> {
            Box::pin(async move {
                if category == StatisticCategory::Custom
                    && stat_id == CustomStatistic::TradedWithVillager as i32
                {
                    self.traded.fetch_add(amount, Ordering::Relaxed);
                }
            })
        }
    }

    fn inventories() -> (Arc<PlayerInventory>, Arc<SimpleInventory>) {
        (
            Arc::new(PlayerInventory::new(
                Arc::new(Mutex::new(EntityEquipment::new())),
                Arc::new(HashMap::new()),
            )),
            Arc::new(SimpleInventory::new(3)),
        )
    }

    #[tokio::test]
    async fn selecting_bookshelf_trade_moves_payment_and_creates_result() {
        let (player_inventory, merchant_inventory) = inventories();
        *player_inventory.main_inventory[0].lock().await = ItemStack::new(12, &Item::EMERALD);
        let mut handler = MerchantScreenHandler::new(
            1,
            &player_inventory,
            merchant_inventory.clone(),
            vec![bookshelf_offer()],
        )
        .await;

        handler.set_selected_offer(0).await;

        let payment = merchant_inventory.get_stack(0).await;
        assert_eq!(payment.lock().await.item_count, 12);
        let result = merchant_inventory.get_stack(2).await;
        assert_eq!(result.lock().await.item.id, Item::BOOKSHELF.id);
        assert!(player_inventory.main_inventory[0].lock().await.is_empty());
    }

    #[tokio::test]
    async fn taking_result_commits_payment_after_delivery() {
        let (player_inventory, merchant_inventory) = inventories();
        merchant_inventory
            .set_stack(0, ItemStack::new(12, &Item::EMERALD))
            .await;
        let player = TestPlayer::new(player_inventory.clone());
        let mut handler = MerchantScreenHandler::new(
            1,
            &player_inventory,
            merchant_inventory.clone(),
            vec![bookshelf_offer()],
        )
        .await;
        let trade_count = Arc::new(AtomicUsize::new(0));
        handler.on_trade = Some(Box::new({
            let trade_count = trade_count.clone();
            move |_| {
                let trade_count = trade_count.clone();
                Box::pin(async move {
                    trade_count.fetch_add(1, Ordering::Relaxed);
                })
            }
        }));
        handler.update_result_slot().await;

        handler
            .on_slot_click(2, 0, SlotActionType::Pickup, &player)
            .await;

        assert_eq!(handler.offers[0].uses, 1);
        assert_eq!(
            merchant_inventory
                .get_stack(0)
                .await
                .lock()
                .await
                .item_count,
            3
        );
        assert_eq!(
            handler.get_behaviour().cursor_stack.lock().await.item.id,
            Item::BOOKSHELF.id
        );
        assert_eq!(player.traded.load(Ordering::Relaxed), 1);
        assert_eq!(trade_count.load(Ordering::Relaxed), 1);
        assert_eq!(player.experience.load(Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn selection_hint_zero_resolves_matching_offer() {
        let (player_inventory, merchant_inventory) = inventories();
        merchant_inventory
            .set_stack(0, ItemStack::new(64, &Item::PAPER))
            .await;
        let player = TestPlayer::new(player_inventory.clone());
        let mut handler = MerchantScreenHandler::new(
            1,
            &player_inventory,
            merchant_inventory.clone(),
            vec![
                bookshelf_offer(),
                single_cost_offer(&Item::PAPER, 24, &Item::EMERALD, 16),
            ],
        )
        .await;
        let traded_offer = Arc::new(AtomicUsize::new(usize::MAX));
        handler.on_trade = Some(Box::new({
            let traded_offer = traded_offer.clone();
            move |offer_index| {
                let traded_offer = traded_offer.clone();
                Box::pin(async move {
                    traded_offer.store(offer_index, Ordering::Relaxed);
                })
            }
        }));
        handler.update_result_slot().await;

        handler
            .on_slot_click(2, 0, SlotActionType::Pickup, &player)
            .await;

        assert_eq!(handler.offers[0].uses, 0);
        assert_eq!(handler.offers[1].uses, 1);
        assert_eq!(traded_offer.load(Ordering::Relaxed), 1);
        assert_eq!(
            merchant_inventory
                .get_stack(0)
                .await
                .lock()
                .await
                .item_count,
            40
        );
        assert_eq!(
            handler.get_behaviour().cursor_stack.lock().await.item.id,
            Item::EMERALD.id
        );
    }

    #[tokio::test]
    async fn invalid_payment_does_not_notify_the_merchant() {
        let (player_inventory, merchant_inventory) = inventories();
        merchant_inventory
            .set_stack(0, ItemStack::new(8, &Item::EMERALD))
            .await;
        let player = TestPlayer::new(player_inventory.clone());
        let mut handler = MerchantScreenHandler::new(
            1,
            &player_inventory,
            merchant_inventory.clone(),
            vec![bookshelf_offer()],
        )
        .await;
        let trade_count = Arc::new(AtomicUsize::new(0));
        handler.on_trade = Some(Box::new({
            let trade_count = trade_count.clone();
            move |_| {
                let trade_count = trade_count.clone();
                Box::pin(async move {
                    trade_count.fetch_add(1, Ordering::Relaxed);
                })
            }
        }));

        handler
            .on_slot_click(2, 0, SlotActionType::Pickup, &player)
            .await;

        assert_eq!(handler.offers[0].uses, 0);
        assert_eq!(trade_count.load(Ordering::Relaxed), 0);
        assert_eq!(player.traded.load(Ordering::Relaxed), 0);
        assert_eq!(
            merchant_inventory
                .get_stack(0)
                .await
                .lock()
                .await
                .item_count,
            8
        );
        assert!(handler.get_behaviour().cursor_stack.lock().await.is_empty());
    }

    #[tokio::test]
    async fn full_inventory_does_not_commit_quick_moved_trade() {
        let (player_inventory, merchant_inventory) = inventories();
        for slot in &player_inventory.main_inventory {
            *slot.lock().await = ItemStack::new(64, &Item::COBBLESTONE);
        }
        merchant_inventory
            .set_stack(0, ItemStack::new(9, &Item::EMERALD))
            .await;
        let player = TestPlayer::new(player_inventory.clone());
        let mut handler = MerchantScreenHandler::new(
            1,
            &player_inventory,
            merchant_inventory.clone(),
            vec![bookshelf_offer()],
        )
        .await;
        handler.update_result_slot().await;

        handler
            .on_slot_click(2, 0, SlotActionType::QuickMove, &player)
            .await;

        assert_eq!(handler.offers[0].uses, 0);
        assert_eq!(
            merchant_inventory
                .get_stack(0)
                .await
                .lock()
                .await
                .item_count,
            9
        );
        assert_eq!(
            merchant_inventory.get_stack(2).await.lock().await.item.id,
            Item::BOOKSHELF.id
        );
        assert_eq!(player.traded.load(Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn quick_move_repeats_while_payment_and_stock_remain() {
        let (player_inventory, merchant_inventory) = inventories();
        merchant_inventory
            .set_stack(0, ItemStack::new(64, &Item::EMERALD))
            .await;
        let player = TestPlayer::new(player_inventory.clone());
        let mut handler = MerchantScreenHandler::new(
            1,
            &player_inventory,
            merchant_inventory.clone(),
            vec![bookshelf_offer()],
        )
        .await;
        handler.update_result_slot().await;

        handler
            .on_slot_click(2, 0, SlotActionType::QuickMove, &player)
            .await;

        assert_eq!(handler.offers[0].uses, 7);
        assert_eq!(player.traded.load(Ordering::Relaxed), 7);
        assert_eq!(
            merchant_inventory
                .get_stack(0)
                .await
                .lock()
                .await
                .item_count,
            1
        );
        assert_eq!(
            player_inventory.main_inventory[8].lock().await.item_count,
            7
        );
    }

    #[tokio::test]
    async fn swapped_payment_slots_are_accepted_and_consumed() {
        let (player_inventory, merchant_inventory) = inventories();
        let mut offer = bookshelf_offer();
        offer.base_cost_a = ItemStackSerializer(Cow::Owned(ItemStack::new(5, &Item::EMERALD)));
        offer.cost_b = Some(ItemStackSerializer(Cow::Owned(ItemStack::new(
            1,
            &Item::BOOK,
        ))));
        merchant_inventory
            .set_stack(0, ItemStack::new(1, &Item::BOOK))
            .await;
        merchant_inventory
            .set_stack(1, ItemStack::new(5, &Item::EMERALD))
            .await;
        let player = TestPlayer::new(player_inventory.clone());
        let mut handler = MerchantScreenHandler::new(
            1,
            &player_inventory,
            merchant_inventory.clone(),
            vec![offer],
        )
        .await;
        handler.update_result_slot().await;

        handler
            .on_slot_click(2, 0, SlotActionType::Pickup, &player)
            .await;

        assert_eq!(handler.offers[0].uses, 1);
        assert!(
            merchant_inventory
                .get_stack(0)
                .await
                .lock()
                .await
                .is_empty()
        );
        assert!(
            merchant_inventory
                .get_stack(1)
                .await
                .lock()
                .await
                .is_empty()
        );
    }
}
