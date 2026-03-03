use std::{
    any::Any,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicI32, AtomicU8, Ordering},
    },
};

use pumpkin_data::screen::WindowType;
use pumpkin_world::{
    inventory::{Clearable, Inventory, InventoryFuture, split_stack},
    item::ItemStack,
};
use tokio::sync::Mutex;

use crate::{
    player::player_inventory::PlayerInventory,
    screen_handler::{
        InventoryPlayer, ItemStackFuture, ScreenHandler, ScreenHandlerBehaviour,
        ScreenHandlerFuture,
    },
    slot::{BoxFuture, NormalSlot, Slot},
};

/// A simple 3-slot inventory for the merchant screen (input1, input2, output).
pub struct MerchantInventory {
    pub items: Vec<Arc<Mutex<ItemStack>>>,
}

impl Default for MerchantInventory {
    fn default() -> Self {
        Self {
            items: vec![
                Arc::new(Mutex::new(ItemStack::EMPTY.clone())),
                Arc::new(Mutex::new(ItemStack::EMPTY.clone())),
                Arc::new(Mutex::new(ItemStack::EMPTY.clone())),
            ],
        }
    }
}

impl MerchantInventory {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Inventory for MerchantInventory {
    fn size(&self) -> usize {
        3
    }

    fn is_empty(&self) -> InventoryFuture<'_, bool> {
        Box::pin(async move {
            for slot in &self.items {
                if !slot.lock().await.is_empty() {
                    return false;
                }
            }
            true
        })
    }

    fn get_stack(&self, slot: usize) -> InventoryFuture<'_, Arc<Mutex<ItemStack>>> {
        Box::pin(async move { self.items[slot].clone() })
    }

    fn remove_stack(&self, slot: usize) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            let mut removed = ItemStack::EMPTY.clone();
            let mut guard = self.items[slot].lock().await;
            std::mem::swap(&mut removed, &mut *guard);
            removed
        })
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move { split_stack(&self.items, slot, amount).await })
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            *self.items[slot].lock().await = stack;
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Clearable for MerchantInventory {
    fn clear(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            for item in &self.items {
                *item.lock().await = ItemStack::EMPTY.clone();
            }
        })
    }
}

/// A snapshot of a trade offer stored on the screen handler for trade selection.
#[derive(Clone)]
pub struct MerchantTradeOffer {
    pub input1_item: u16,
    pub input1_count: u8,
    pub input2_item: Option<u16>,
    pub input2_count: u8,
    pub output: ItemStack,
    pub max_uses: i32,
    pub uses: i32,
}

/// Shared state between `MerchantScreenHandler` and `MerchantOutputSlot`,
/// allowing the output slot to validate and consume trades on any extraction path
/// (Pickup, Swap, Throw, PickupAll, QuickCraft).
pub struct MerchantTradeState {
    pub trade_offers: Mutex<Vec<MerchantTradeOffer>>,
    pub selected_trade: AtomicI32,
    pub inventory: Arc<MerchantInventory>,
    /// Channel to notify the villager entity when a trade completes (sends trade index).
    trade_completion_tx: std::sync::Mutex<Option<tokio::sync::mpsc::UnboundedSender<usize>>>,
}

impl MerchantTradeState {
    /// Check if the current input slots satisfy the selected trade's requirements.
    pub async fn can_fulfill_trade(&self) -> bool {
        let selected = self.selected_trade.load(Ordering::Acquire);
        if selected < 0 {
            return false;
        }
        let idx = selected as usize;
        let offers = self.trade_offers.lock().await;
        if idx >= offers.len() {
            return false;
        }
        let offer = &offers[idx];
        if offer.uses >= offer.max_uses {
            return false;
        }

        let input1 = self.inventory.items[0].lock().await;
        if input1.is_empty() || input1.item.id != offer.input1_item {
            return false;
        }
        if input1.item_count < offer.input1_count {
            return false;
        }
        drop(input1);

        if let Some(input2_item) = offer.input2_item {
            let input2 = self.inventory.items[1].lock().await;
            if input2.is_empty() || input2.item.id != input2_item {
                return false;
            }
            if input2.item_count < offer.input2_count {
                return false;
            }
        }

        true
    }

    /// Consume input items and complete the trade (increment uses, notify villager).
    pub async fn on_trade_taken(&self) {
        let selected = self.selected_trade.load(Ordering::Acquire);
        if selected < 0 {
            return;
        }
        let idx = selected as usize;

        // Read offer requirements then drop the lock before modifying inventory
        let (input1_count, input2_item, input2_count) = {
            let offers = self.trade_offers.lock().await;
            if idx >= offers.len() {
                return;
            }
            (
                offers[idx].input1_count,
                offers[idx].input2_item,
                offers[idx].input2_count,
            )
        };

        // Consume input1
        {
            let mut input1 = self.inventory.items[0].lock().await;
            if input1.item_count <= input1_count {
                *input1 = ItemStack::EMPTY.clone();
            } else {
                input1.decrement(input1_count);
            }
        }

        // Consume input2 if required
        if input2_item.is_some() {
            let mut input2 = self.inventory.items[1].lock().await;
            if input2.item_count <= input2_count {
                *input2 = ItemStack::EMPTY.clone();
            } else {
                input2.decrement(input2_count);
            }
        }

        // Increment uses on the snapshot
        {
            let mut offers = self.trade_offers.lock().await;
            if idx < offers.len() {
                offers[idx].uses += 1;
            }
        }

        // Notify villager entity via channel
        if let Ok(guard) = self.trade_completion_tx.lock() {
            if let Some(ref tx) = *guard {
                let _ = tx.send(idx);
            }
        }
    }
}

/// Output slot for merchant GUI — prevents insertion and validates trades on extraction.
pub struct MerchantOutputSlot {
    pub inventory: Arc<dyn Inventory>,
    pub index: usize,
    pub id: AtomicU8,
    pub trade_state: Arc<MerchantTradeState>,
}

impl MerchantOutputSlot {
    pub fn new(
        inventory: Arc<dyn Inventory>,
        index: usize,
        trade_state: Arc<MerchantTradeState>,
    ) -> Self {
        Self {
            inventory,
            index,
            id: AtomicU8::new(0),
            trade_state,
        }
    }
}

impl Slot for MerchantOutputSlot {
    fn get_inventory(&self) -> Arc<dyn Inventory> {
        self.inventory.clone()
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_id(&self, id: usize) {
        self.id.store(id as u8, Ordering::Relaxed);
    }

    fn can_insert<'a>(&'a self, _stack: &'a ItemStack) -> BoxFuture<'a, bool> {
        // Players cannot insert items into the output slot
        Box::pin(async { false })
    }

    fn can_take_items(&self, _player: &dyn InventoryPlayer) -> BoxFuture<'_, bool> {
        // Only allow extraction if the selected trade can be fulfilled
        Box::pin(async move { self.trade_state.can_fulfill_trade().await })
    }

    fn on_take_item<'a>(
        &'a self,
        _player: &'a dyn InventoryPlayer,
        _stack: &'a ItemStack,
    ) -> BoxFuture<'a, ()> {
        // Consume inputs, increment uses, and notify the villager entity
        Box::pin(async move {
            self.trade_state.on_trade_taken().await;
            self.mark_dirty().await;
        })
    }

    fn mark_dirty(&self) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.inventory.mark_dirty();
        })
    }
}

/// Screen handler for the Merchant (villager trading) GUI.
/// Slot layout: 0=input1, 1=input2, 2=output, then 3..38=player inventory (27+9)
pub struct MerchantScreenHandler {
    pub trade_state: Arc<MerchantTradeState>,
    pub villager_entity_id: i32,
    /// Reference to the villager's `trading_player_id` for cleanup on close.
    pub villager_trading_lock: Option<Arc<std::sync::atomic::AtomicI32>>,
    behaviour: ScreenHandlerBehaviour,
}

impl MerchantScreenHandler {
    pub fn new(
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        trade_completion_tx: Option<tokio::sync::mpsc::UnboundedSender<usize>>,
    ) -> Self {
        let inventory = Arc::new(MerchantInventory::new());
        let trade_state = Arc::new(MerchantTradeState {
            trade_offers: Mutex::new(Vec::new()),
            selected_trade: AtomicI32::new(-1),
            inventory: inventory.clone(),
            trade_completion_tx: std::sync::Mutex::new(trade_completion_tx),
        });

        let mut handler = Self {
            trade_state: trade_state.clone(),
            villager_entity_id: -1,
            villager_trading_lock: None,
            behaviour: ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Merchant)),
        };

        // Add 2 input slots (normal) + 1 output slot (validated extraction)
        let inv: Arc<dyn Inventory> = inventory;
        handler.add_slot(Arc::new(NormalSlot::new(inv.clone(), 0)));
        handler.add_slot(Arc::new(NormalSlot::new(inv.clone(), 1)));
        handler.add_slot(Arc::new(MerchantOutputSlot::new(inv, 2, trade_state)));

        // Add player inventory + hotbar slots
        let player_inv: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&player_inv);

        handler
    }

    /// Set the trade offers snapshot (called after creating the handler).
    pub async fn set_trade_offers(&self, offers: Vec<MerchantTradeOffer>) {
        *self.trade_state.trade_offers.lock().await = offers;
    }

    /// Select a trade by index and update the output slot preview.
    pub async fn select_trade(&self, index: i32) {
        self.trade_state
            .selected_trade
            .store(index, Ordering::Release);
        let offers = self.trade_state.trade_offers.lock().await;
        if index >= 0 && (index as usize) < offers.len() {
            let offer = &offers[index as usize];
            if offer.uses < offer.max_uses {
                self.trade_state
                    .inventory
                    .set_stack(2, offer.output.clone())
                    .await;
            } else {
                self.trade_state
                    .inventory
                    .set_stack(2, ItemStack::EMPTY.clone())
                    .await;
            }
        } else {
            self.trade_state
                .inventory
                .set_stack(2, ItemStack::EMPTY.clone())
                .await;
        }
    }
}

impl ScreenHandler for MerchantScreenHandler {
    fn on_closed<'a>(&'a mut self, player: &'a dyn InventoryPlayer) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            // Return input items to the player (slots 0 and 1)
            for slot_idx in 0..2 {
                let stack = self.trade_state.inventory.remove_stack(slot_idx).await;
                if !stack.is_empty() {
                    player.drop_item(stack, false).await;
                }
            }
            // Clear output
            self.trade_state
                .inventory
                .set_stack(2, ItemStack::EMPTY.clone())
                .await;
            // Reset villager trading state
            if let Some(ref trading_lock) = self.villager_trading_lock {
                trading_lock.store(-1, std::sync::atomic::Ordering::Relaxed);
            }
            self.default_on_closed(player).await;
        })
    }

    fn handle_slot_click<'a>(
        &'a self,
        _player: &'a dyn InventoryPlayer,
        _click_type: crate::container_click::MouseClick,
        _slot: Arc<dyn Slot>,
        _slot_stack: ItemStack,
        _cursor_stack: ItemStack,
    ) -> ScreenHandlerFuture<'a, bool> {
        // Trade validation is handled by MerchantOutputSlot::can_take_items/on_take_item,
        // which covers ALL extraction paths (Pickup, Swap, Throw, PickupAll, QuickCraft).
        Box::pin(async move { false })
    }

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

                if slot_index == 2 {
                    // Output slot: validate and consume trade before moving
                    if !self.trade_state.can_fulfill_trade().await {
                        return ItemStack::EMPTY.clone();
                    }
                    self.trade_state.on_trade_taken().await;
                    if !self.insert_item(&mut slot_stack_mut, 3, 39, true).await {
                        return ItemStack::EMPTY.clone();
                    }
                } else if slot_index < 3 {
                    // Move from input slots to player inventory
                    if !self.insert_item(&mut slot_stack_mut, 3, 39, true).await {
                        return ItemStack::EMPTY.clone();
                    }
                } else {
                    // Move from player inventory to merchant input slots (0..2)
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
}
