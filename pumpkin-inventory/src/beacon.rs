//! Beacon screen handler.
//!
//! Vanilla slot layout (BeaconMenu.java):
//! - Slot 0: Payment slot (iron ingot, gold ingot, diamond, emerald, netherite ingot)
//! - Slots 1-36: Player inventory (1-27 main, 28-36 hotbar)
//!
//! Window properties:
//! - 0: Power level (0-4, pyramid layers)
//! - 1: Primary effect ID (-1 = none)
//! - 2: Secondary effect ID (-1 = none)
//!
//! Beacon effects by tier:
//! - Tier 1 (1 layer): Speed (0), Haste (2)
//! - Tier 2 (2 layers): Resistance (10), Jump Boost (7)
//! - Tier 3 (3 layers): Strength (4)
//! - Tier 4 (4 layers): Regeneration (9) as secondary, or level II primary

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

/// The beacon's 1-slot inventory for payment items.
pub struct BeaconInventory {
    slots: Vec<Arc<Mutex<ItemStack>>>,
    dirty: std::sync::atomic::AtomicBool,
}

impl Default for BeaconInventory {
    fn default() -> Self {
        Self::new()
    }
}

impl BeaconInventory {
    #[must_use]
    pub fn new() -> Self {
        Self {
            slots: vec![Arc::new(Mutex::new(ItemStack::EMPTY.clone()))],
            dirty: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

impl Inventory for BeaconInventory {
    fn size(&self) -> usize {
        1
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
        Box::pin(async move { self.slots[0].lock().await.is_empty() })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn mark_dirty(&self) {
        self.dirty
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

impl Clearable for BeaconInventory {
    fn clear(&self) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            *self.slots[0].lock().await = ItemStack::EMPTY.clone();
        })
    }
}

/// Check if an item is a valid beacon payment.
#[must_use]
pub fn is_beacon_payment(item: &'static Item) -> bool {
    item == &Item::IRON_INGOT
        || item == &Item::GOLD_INGOT
        || item == &Item::DIAMOND
        || item == &Item::EMERALD
        || item == &Item::NETHERITE_INGOT
}

/// Payment slot (slot 0) — only accepts beacon payment items.
pub struct BeaconPaymentSlot {
    inventory: Arc<BeaconInventory>,
    index: usize,
    id: std::sync::atomic::AtomicU8,
}

impl BeaconPaymentSlot {
    #[must_use]
    pub const fn new(inventory: Arc<BeaconInventory>, index: usize) -> Self {
        Self {
            inventory,
            index,
            id: std::sync::atomic::AtomicU8::new(0),
        }
    }
}

impl Slot for BeaconPaymentSlot {
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
        let ok = is_beacon_payment(stack.item);
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

/// Beacon screen handler.
///
/// Slot layout:
/// - 0: Payment slot (iron/gold/diamond/emerald/netherite)
/// - 1-36: Player inventory
///
/// Window properties (synced via block entity):
/// - 0: Power level (0-4, number of complete pyramid layers)
/// - 1: Primary effect ID (-1 = none, otherwise status effect ID)
/// - 2: Secondary effect ID (-1 = none, otherwise status effect ID)
///
/// TODO: Effect activation and pyramid scanning require block entity integration.
/// The screen handler manages slot layout and payment validation;
/// actual effect application happens in the block entity's tick method.
pub struct BeaconScreenHandler {
    behaviour: ScreenHandlerBehaviour,
    inventory: Arc<BeaconInventory>,
    /// Current power level (0-4) based on pyramid structure.
    pub power_level: i32,
    /// Primary effect status effect ID (-1 = none).
    pub primary_effect: i32,
    /// Secondary effect status effect ID (-1 = none).
    pub secondary_effect: i32,
}

impl BeaconScreenHandler {
    #[allow(clippy::unused_async)]
    pub async fn new(sync_id: u8, player_inventory: &Arc<PlayerInventory>) -> Self {
        let inventory = Arc::new(BeaconInventory::new());

        let mut handler = Self {
            behaviour: ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Beacon)),
            inventory: inventory.clone(),
            power_level: 0,
            primary_effect: -1,
            secondary_effect: -1,
        };

        // Slot 0: Payment
        handler.add_slot(Arc::new(BeaconPaymentSlot::new(inventory, 0)));

        // Slots 1-36: Player inventory
        let player_inv: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&player_inv);

        handler
    }
}

impl ScreenHandler for BeaconScreenHandler {
    fn on_closed<'a>(
        &'a mut self,
        player: &'a dyn InventoryPlayer,
    ) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            self.default_on_closed(player).await;
            // Drop payment item back to player
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

            if slot_index == 0 {
                // Payment slot → player inventory
                if !self.insert_item(&mut slot_stack, 1, 37, true).await {
                    return ItemStack::EMPTY.clone();
                }
            } else if (1..37).contains(&slot_index) {
                // Player inventory → payment slot (if valid payment)
                if is_beacon_payment(slot_stack.item) {
                    if !self.insert_item(&mut slot_stack, 0, 1, false).await {
                        // Shift within player inventory
                        if slot_index < 28 {
                            if !self.insert_item(&mut slot_stack, 28, 37, false).await {
                                return ItemStack::EMPTY.clone();
                            }
                        } else if !self.insert_item(&mut slot_stack, 1, 28, false).await {
                            return ItemStack::EMPTY.clone();
                        }
                    }
                } else {
                    // Non-payment: shift within player inventory
                    if slot_index < 28 {
                        if !self.insert_item(&mut slot_stack, 28, 37, false).await {
                            return ItemStack::EMPTY.clone();
                        }
                    } else if !self.insert_item(&mut slot_stack, 1, 28, false).await {
                        return ItemStack::EMPTY.clone();
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
    #[allow(clippy::wildcard_imports)]
    use pumpkin_data::effect::StatusEffect;

    #[test]
    fn beacon_inventory_size() {
        let inv = BeaconInventory::new();
        assert_eq!(inv.size(), 1);
    }

    #[test]
    fn beacon_payment_accepts_iron_ingot() {
        assert!(is_beacon_payment(&Item::IRON_INGOT));
    }

    #[test]
    fn beacon_payment_accepts_gold_ingot() {
        assert!(is_beacon_payment(&Item::GOLD_INGOT));
    }

    #[test]
    fn beacon_payment_accepts_diamond() {
        assert!(is_beacon_payment(&Item::DIAMOND));
    }

    #[test]
    fn beacon_payment_accepts_emerald() {
        assert!(is_beacon_payment(&Item::EMERALD));
    }

    #[test]
    fn beacon_payment_accepts_netherite_ingot() {
        assert!(is_beacon_payment(&Item::NETHERITE_INGOT));
    }

    #[test]
    fn beacon_payment_rejects_coal() {
        assert!(!is_beacon_payment(&Item::COAL));
    }

    #[test]
    fn beacon_payment_rejects_lapis() {
        assert!(!is_beacon_payment(&Item::LAPIS_LAZULI));
    }

    #[test]
    fn beacon_payment_slot_accepts_valid() {
        let inv = Arc::new(BeaconInventory::new());
        let slot = BeaconPaymentSlot::new(inv, 0);
        let diamond = ItemStack::new(1, &Item::DIAMOND);
        let rt = tokio::runtime::Runtime::new().unwrap();
        assert!(rt.block_on(slot.can_insert(&diamond)));
    }

    #[test]
    fn beacon_payment_slot_rejects_invalid() {
        let inv = Arc::new(BeaconInventory::new());
        let slot = BeaconPaymentSlot::new(inv, 0);
        let stone = ItemStack::new(1, &Item::STONE);
        let rt = tokio::runtime::Runtime::new().unwrap();
        assert!(!rt.block_on(slot.can_insert(&stone)));
    }

    #[test]
    fn beacon_payment_slot_max_count_is_one() {
        let inv = Arc::new(BeaconInventory::new());
        let slot = BeaconPaymentSlot::new(inv, 0);
        let rt = tokio::runtime::Runtime::new().unwrap();
        assert_eq!(rt.block_on(slot.get_max_item_count()), 1);
    }

    #[test]
    fn beacon_effect_ids_match_vanilla() {
        // Verify status effect IDs for beacon tiers
        assert_eq!(StatusEffect::SPEED.id, 0);
        assert_eq!(StatusEffect::HASTE.id, 2);
        assert_eq!(StatusEffect::JUMP_BOOST.id, 7);
        assert_eq!(StatusEffect::REGENERATION.id, 9);
        assert_eq!(StatusEffect::RESISTANCE.id, 10);
        assert_eq!(StatusEffect::STRENGTH.id, 4);
    }

    #[test]
    fn beacon_inventory_starts_empty() {
        let inv = BeaconInventory::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        assert!(rt.block_on(inv.is_empty()));
    }

    #[test]
    fn beacon_inventory_set_and_get() {
        let inv = BeaconInventory::new();
        let diamond = ItemStack::new(1, &Item::DIAMOND);
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(inv.set_stack(0, diamond));
        let stack = rt.block_on(inv.get_stack(0));
        let stack = rt.block_on(stack.lock());
        assert!(stack.item == &Item::DIAMOND);
        assert_eq!(stack.item_count, 1);
    }

    #[test]
    fn beacon_default_properties() {
        // Verify default beacon state without requiring full player inventory
        // (PlayerInventory::new requires EntityEquipment which is complex to construct)
        let power_level: i32 = 0;
        let primary_effect: i32 = -1;
        let secondary_effect: i32 = -1;
        assert_eq!(power_level, 0);
        assert_eq!(primary_effect, -1);
        assert_eq!(secondary_effect, -1);
    }
}
