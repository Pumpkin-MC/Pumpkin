//! Brewing stand screen handler.
//!
//! Vanilla slot layout (BrewingStandMenu.java):
//! - Slots 0-2: Potion bottle slots (output positions)
//! - Slot 3: Ingredient slot (reagent)
//! - Slot 4: Fuel slot (blaze powder)
//! - Slots 5-40: Player inventory (5-31 main, 32-40 hotbar)

use std::{any::Any, sync::Arc};

use pumpkin_data::item::Item;
use pumpkin_data::screen::WindowType;
use pumpkin_world::inventory::{Clearable, Inventory, InventoryFuture, split_stack};
use pumpkin_world::item::ItemStack;
use tokio::sync::Mutex;

use crate::player::player_inventory::PlayerInventory;
use crate::screen_handler::{
    InventoryPlayer, ItemStackFuture, ScreenHandler, ScreenHandlerBehaviour, ScreenHandlerFuture,
};
use crate::slot::{BoxFuture, Slot};

/// The brewing stand's 5-slot inventory (3 bottles + ingredient + fuel).
pub struct BrewingStandInventory {
    slots: Vec<Arc<Mutex<ItemStack>>>,
    dirty: std::sync::atomic::AtomicBool,
}

impl Default for BrewingStandInventory {
    fn default() -> Self {
        Self::new()
    }
}

impl BrewingStandInventory {
    #[must_use]
    pub fn new() -> Self {
        Self {
            slots: (0..5)
                .map(|_| Arc::new(Mutex::new(ItemStack::EMPTY.clone())))
                .collect(),
            dirty: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

impl Inventory for BrewingStandInventory {
    fn size(&self) -> usize {
        5
    }

    fn get_stack(&self, slot: usize) -> InventoryFuture<'_, Arc<Mutex<ItemStack>>> {
        Box::pin(async move { self.slots[slot].clone() })
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            *self.slots[slot].lock().await = stack;
        })
    }

    fn remove_stack(&self, slot: usize) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            let mut removed = ItemStack::EMPTY.clone();
            let mut guard = self.slots[slot].lock().await;
            std::mem::swap(&mut removed, &mut *guard);
            removed
        })
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move { split_stack(&self.slots, slot, amount).await })
    }

    fn is_empty(&self) -> InventoryFuture<'_, bool> {
        Box::pin(async move {
            for slot in &self.slots {
                if !slot.lock().await.is_empty() {
                    return false;
                }
            }
            true
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn mark_dirty(&self) {
        self.dirty
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

impl Clearable for BrewingStandInventory {
    fn clear(&self) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            for slot in &self.slots {
                *slot.lock().await = ItemStack::EMPTY.clone();
            }
        })
    }
}

/// Potion bottle slot (slots 0-2) — accepts potions, splash potions, lingering potions,
/// and glass bottles.
pub struct PotionSlot {
    inventory: Arc<BrewingStandInventory>,
    index: usize,
    id: std::sync::atomic::AtomicU8,
}

impl PotionSlot {
    #[must_use]
    pub const fn new(inventory: Arc<BrewingStandInventory>, index: usize) -> Self {
        Self {
            inventory,
            index,
            id: std::sync::atomic::AtomicU8::new(0),
        }
    }
}

/// Check if an item can go into a potion bottle slot.
#[must_use]
pub fn is_potion_slot_item(item: &'static Item) -> bool {
    item == &Item::POTION
        || item == &Item::SPLASH_POTION
        || item == &Item::LINGERING_POTION
        || item == &Item::GLASS_BOTTLE
}

impl Slot for PotionSlot {
    fn get_inventory(&self) -> Arc<dyn Inventory> {
        self.inventory.clone()
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_id(&self, id: usize) {
        self.id
            .store(id as u8, std::sync::atomic::Ordering::Relaxed);
    }

    fn can_insert(&self, stack: &ItemStack) -> BoxFuture<'_, bool> {
        let ok = is_potion_slot_item(stack.item);
        Box::pin(async move { ok })
    }

    fn get_stack(&self) -> BoxFuture<'_, Arc<Mutex<ItemStack>>> {
        Box::pin(async move { self.inventory.get_stack(self.index).await })
    }

    fn get_cloned_stack(&self) -> BoxFuture<'_, ItemStack> {
        Box::pin(async move {
            let stack = self.inventory.get_stack(self.index).await;
            stack.lock().await.clone()
        })
    }

    fn has_stack(&self) -> BoxFuture<'_, bool> {
        Box::pin(async move {
            let stack = self.inventory.get_stack(self.index).await;
            !stack.lock().await.is_empty()
        })
    }

    fn set_stack(&self, stack: ItemStack) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.inventory.set_stack(self.index, stack).await;
        })
    }

    fn set_stack_prev(&self, stack: ItemStack, _previous: ItemStack) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.inventory.set_stack(self.index, stack).await;
        })
    }

    fn mark_dirty(&self) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.inventory.mark_dirty();
        })
    }

    fn get_max_item_count(&self) -> BoxFuture<'_, u8> {
        Box::pin(async move { 1 })
    }

    fn take_stack(&self, amount: u8) -> BoxFuture<'_, ItemStack> {
        Box::pin(async move { self.inventory.remove_stack_specific(self.index, amount).await })
    }
}

/// Fuel slot (slot 4) — only accepts blaze powder.
pub struct FuelSlot {
    inventory: Arc<BrewingStandInventory>,
    index: usize,
    id: std::sync::atomic::AtomicU8,
}

impl FuelSlot {
    #[must_use]
    pub const fn new(inventory: Arc<BrewingStandInventory>, index: usize) -> Self {
        Self {
            inventory,
            index,
            id: std::sync::atomic::AtomicU8::new(0),
        }
    }
}

impl Slot for FuelSlot {
    fn get_inventory(&self) -> Arc<dyn Inventory> {
        self.inventory.clone()
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_id(&self, id: usize) {
        self.id
            .store(id as u8, std::sync::atomic::Ordering::Relaxed);
    }

    fn can_insert(&self, stack: &ItemStack) -> BoxFuture<'_, bool> {
        let ok = stack.item == &Item::BLAZE_POWDER;
        Box::pin(async move { ok })
    }

    fn get_stack(&self) -> BoxFuture<'_, Arc<Mutex<ItemStack>>> {
        Box::pin(async move { self.inventory.get_stack(self.index).await })
    }

    fn get_cloned_stack(&self) -> BoxFuture<'_, ItemStack> {
        Box::pin(async move {
            let stack = self.inventory.get_stack(self.index).await;
            stack.lock().await.clone()
        })
    }

    fn has_stack(&self) -> BoxFuture<'_, bool> {
        Box::pin(async move {
            let stack = self.inventory.get_stack(self.index).await;
            !stack.lock().await.is_empty()
        })
    }

    fn set_stack(&self, stack: ItemStack) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.inventory.set_stack(self.index, stack).await;
        })
    }

    fn set_stack_prev(&self, stack: ItemStack, _previous: ItemStack) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.inventory.set_stack(self.index, stack).await;
        })
    }

    fn mark_dirty(&self) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.inventory.mark_dirty();
        })
    }

    fn get_max_item_count(&self) -> BoxFuture<'_, u8> {
        Box::pin(async move { 64 })
    }

    fn take_stack(&self, amount: u8) -> BoxFuture<'_, ItemStack> {
        Box::pin(async move { self.inventory.remove_stack_specific(self.index, amount).await })
    }
}

/// Brewing stand screen handler.
///
/// Slot layout:
/// - 0-2: Potion bottle slots
/// - 3: Ingredient slot
/// - 4: Fuel slot (blaze powder)
/// - 5-40: Player inventory
///
/// Window properties:
/// - 0: Brew time (0-400 ticks)
/// - 1: Fuel time (remaining fuel uses)
///
/// TODO: Actual brewing logic is tick-driven and requires block entity integration.
/// The screen handler manages slot layout and transfers; brewing processing
/// happens in the block entity's tick method.
pub struct BrewingStandScreenHandler {
    behaviour: ScreenHandlerBehaviour,
    inventory: Arc<BrewingStandInventory>,
    /// Current brew progress (0-400 ticks)
    pub brew_time: i32,
    /// Remaining fuel uses from blaze powder
    pub fuel_time: i32,
}

impl BrewingStandScreenHandler {
    #[allow(clippy::unused_async)]
    pub async fn new(
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
    ) -> Self {
        let inventory = Arc::new(BrewingStandInventory::new());

        let mut handler = Self {
            behaviour: ScreenHandlerBehaviour::new(sync_id, Some(WindowType::BrewingStand)),
            inventory: inventory.clone(),
            brew_time: 0,
            fuel_time: 0,
        };

        // Slots 0-2: Potion bottle slots
        for i in 0..3 {
            handler.add_slot(Arc::new(PotionSlot::new(inventory.clone(), i)));
        }
        // Slot 3: Ingredient
        handler.add_slot(Arc::new(crate::slot::NormalSlot::new(
            inventory.clone(),
            3,
        )));
        // Slot 4: Fuel (blaze powder)
        handler.add_slot(Arc::new(FuelSlot::new(inventory, 4)));

        // Slots 5-40: Player inventory
        let player_inv: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&player_inv);

        handler
    }
}

impl ScreenHandler for BrewingStandScreenHandler {
    fn on_closed<'a>(
        &'a mut self,
        player: &'a dyn InventoryPlayer,
    ) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            self.default_on_closed(player).await;
            // Drop non-result items back to player
            self.drop_inventory(player, self.inventory.clone()).await;
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
        player: &'a dyn InventoryPlayer,
        slot_index: i32,
    ) -> ItemStackFuture<'a> {
        Box::pin(async move {
            let slot = self.get_behaviour().slots[slot_index as usize].clone();

            if !slot.has_stack().await {
                return ItemStack::EMPTY.clone();
            }

            let slot_stack = slot.get_stack().await;
            let mut slot_stack = slot_stack.lock().await;
            let stack_prev = slot_stack.clone();

            if (0..=2).contains(&slot_index) {
                // Potion slots → player inventory
                if !self.insert_item(&mut slot_stack, 5, 41, true).await {
                    return ItemStack::EMPTY.clone();
                }
            } else if slot_index == 3 {
                // Ingredient → player inventory
                if !self.insert_item(&mut slot_stack, 5, 41, true).await {
                    return ItemStack::EMPTY.clone();
                }
            } else if slot_index == 4 {
                // Fuel → player inventory
                if !self.insert_item(&mut slot_stack, 5, 41, true).await {
                    return ItemStack::EMPTY.clone();
                }
            } else if (5..41).contains(&slot_index) {
                // Player inventory → brewing stand
                if is_potion_slot_item(slot_stack.item) {
                    // Try potion slots 0-2
                    if !self.insert_item(&mut slot_stack, 0, 3, false).await {
                        return ItemStack::EMPTY.clone();
                    }
                } else if slot_stack.item == &Item::BLAZE_POWDER {
                    // Try fuel slot
                    if !self.insert_item(&mut slot_stack, 4, 5, false).await {
                        return ItemStack::EMPTY.clone();
                    }
                } else {
                    // Try ingredient slot
                    if !self.insert_item(&mut slot_stack, 3, 4, false).await {
                        // Try within player inventory
                        if slot_index < 32 {
                            if !self.insert_item(&mut slot_stack, 32, 41, false).await {
                                return ItemStack::EMPTY.clone();
                            }
                        } else if !self.insert_item(&mut slot_stack, 5, 32, false).await {
                            return ItemStack::EMPTY.clone();
                        }
                    }
                }
            }

            let stack = slot_stack.clone();
            drop(slot_stack);

            if stack.is_empty() {
                slot.set_stack_prev(ItemStack::EMPTY.clone(), stack_prev.clone())
                    .await;
            } else {
                slot.mark_dirty().await;
            }

            if stack.item_count == stack_prev.item_count {
                return ItemStack::EMPTY.clone();
            }

            slot.on_take_item(player, &stack).await;
            stack_prev
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brewing_inventory_size() {
        let inv = BrewingStandInventory::new();
        assert_eq!(inv.size(), 5);
    }

    #[test]
    fn potion_slot_accepts_potion() {
        let inv = Arc::new(BrewingStandInventory::new());
        let slot = PotionSlot::new(inv, 0);
        let potion = ItemStack::new(1, &Item::POTION);
        let rt = tokio::runtime::Runtime::new().unwrap();
        assert!(rt.block_on(slot.can_insert(&potion)));
    }

    #[test]
    fn potion_slot_accepts_glass_bottle() {
        let inv = Arc::new(BrewingStandInventory::new());
        let slot = PotionSlot::new(inv, 0);
        let bottle = ItemStack::new(1, &Item::GLASS_BOTTLE);
        let rt = tokio::runtime::Runtime::new().unwrap();
        assert!(rt.block_on(slot.can_insert(&bottle)));
    }

    #[test]
    fn potion_slot_rejects_non_potion() {
        let inv = Arc::new(BrewingStandInventory::new());
        let slot = PotionSlot::new(inv, 0);
        let diamond = ItemStack::new(1, &Item::DIAMOND);
        let rt = tokio::runtime::Runtime::new().unwrap();
        assert!(!rt.block_on(slot.can_insert(&diamond)));
    }

    #[test]
    fn fuel_slot_accepts_blaze_powder() {
        let inv = Arc::new(BrewingStandInventory::new());
        let slot = FuelSlot::new(inv, 4);
        let powder = ItemStack::new(1, &Item::BLAZE_POWDER);
        let rt = tokio::runtime::Runtime::new().unwrap();
        assert!(rt.block_on(slot.can_insert(&powder)));
    }

    #[test]
    fn fuel_slot_rejects_non_fuel() {
        let inv = Arc::new(BrewingStandInventory::new());
        let slot = FuelSlot::new(inv, 4);
        let coal = ItemStack::new(1, &Item::COAL);
        let rt = tokio::runtime::Runtime::new().unwrap();
        assert!(!rt.block_on(slot.can_insert(&coal)));
    }

    #[test]
    fn potion_slot_max_count_is_one() {
        let inv = Arc::new(BrewingStandInventory::new());
        let slot = PotionSlot::new(inv, 0);
        let rt = tokio::runtime::Runtime::new().unwrap();
        assert_eq!(rt.block_on(slot.get_max_item_count()), 1);
    }
}
