//! Enchanting table screen handler.
//!
//! Vanilla slot layout (EnchantmentMenu.java):
//! - Slot 0: Item to enchant
//! - Slot 1: Lapis lazuli
//! - Slots 2-37: Player inventory (2-28 main, 29-37 hotbar)

use std::{any::Any, sync::Arc};

use pumpkin_data::item::Item;
use pumpkin_data::screen::WindowType;
use pumpkin_data::tag;
use pumpkin_data::tag::Taggable;
use pumpkin_world::inventory::{Clearable, Inventory, InventoryFuture, split_stack};
use pumpkin_world::item::ItemStack;
use tokio::sync::Mutex;

use crate::player::player_inventory::PlayerInventory;
use crate::screen_handler::{
    InventoryPlayer, ItemStackFuture, ScreenHandler, ScreenHandlerBehaviour, ScreenHandlerFuture,
};
use crate::slot::{BoxFuture, Slot};

/// The enchanting table's 2-slot inventory (item + lapis lazuli).
pub struct EnchantingTableInventory {
    slots: Vec<Arc<Mutex<ItemStack>>>,
    dirty: std::sync::atomic::AtomicBool,
}

impl Default for EnchantingTableInventory {
    fn default() -> Self {
        Self::new()
    }
}

impl EnchantingTableInventory {
    #[must_use]
    pub fn new() -> Self {
        Self {
            slots: vec![
                Arc::new(Mutex::new(ItemStack::EMPTY.clone())),
                Arc::new(Mutex::new(ItemStack::EMPTY.clone())),
            ],
            dirty: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

impl Inventory for EnchantingTableInventory {
    fn size(&self) -> usize {
        2
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

impl Clearable for EnchantingTableInventory {
    fn clear(&self) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            for slot in &self.slots {
                *slot.lock().await = ItemStack::EMPTY.clone();
            }
        })
    }
}

/// Lapis lazuli slot — only accepts lapis lazuli items.
pub struct LapisSlot {
    inventory: Arc<EnchantingTableInventory>,
    index: usize,
    id: std::sync::atomic::AtomicU8,
}

impl LapisSlot {
    #[must_use]
    pub const fn new(inventory: Arc<EnchantingTableInventory>, index: usize) -> Self {
        Self {
            inventory,
            index,
            id: std::sync::atomic::AtomicU8::new(0),
        }
    }
}

impl Slot for LapisSlot {
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
        let is_lapis = stack.item == &Item::LAPIS_LAZULI;
        Box::pin(async move { is_lapis })
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

/// Enchanting table screen handler.
///
/// Slot layout:
/// - 0: Item to enchant
/// - 1: Lapis lazuli
/// - 2-37: Player inventory
///
/// Window properties (10 total):
/// - 0-2: Level requirements for 3 enchantment options
/// - 3: Enchantment seed
/// - 4-6: Enchantment IDs for 3 options
/// - 7-9: Enchantment levels for 3 options
///
/// TODO: Enchantment generation requires bookshelf counting and seed-based randomization,
/// which depends on block entity / world access (not available in pumpkin-inventory).
pub struct EnchantingTableScreenHandler {
    behaviour: ScreenHandlerBehaviour,
    inventory: Arc<EnchantingTableInventory>,
    /// Enchantment seed for randomization
    pub seed: i32,
    /// Level requirements for the 3 enchantment options
    pub level_requirements: [i32; 3],
    /// Enchantment IDs for the 3 options (-1 = none)
    pub enchantment_ids: [i32; 3],
    /// Enchantment levels for the 3 options (0 = none)
    pub enchantment_levels: [i32; 3],
}

impl EnchantingTableScreenHandler {
    #[allow(clippy::unused_async)]
    pub async fn new(
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
    ) -> Self {
        let inventory = Arc::new(EnchantingTableInventory::new());

        let mut handler = Self {
            behaviour: ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Enchantment)),
            inventory: inventory.clone(),
            seed: 0,
            level_requirements: [0; 3],
            enchantment_ids: [-1; 3],
            enchantment_levels: [0; 3],
        };

        // Slot 0: Item to enchant
        handler.add_slot(Arc::new(crate::slot::NormalSlot::new(
            inventory.clone(),
            0,
        )));
        // Slot 1: Lapis lazuli
        handler.add_slot(Arc::new(LapisSlot::new(inventory, 1)));

        // Slots 2-37: Player inventory
        let player_inv: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&player_inv);

        handler
    }
}

impl ScreenHandler for EnchantingTableScreenHandler {
    fn on_closed<'a>(
        &'a mut self,
        player: &'a dyn InventoryPlayer,
    ) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            self.default_on_closed(player).await;
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

            if (0..=1).contains(&slot_index) {
                // Enchanting slots → player inventory
                if !self.insert_item(&mut slot_stack, 2, 38, true).await {
                    return ItemStack::EMPTY.clone();
                }
            } else if (2..38).contains(&slot_index) {
                // Player inventory → enchanting table
                // Try lapis lazuli slot first, then item slot
                if slot_stack.item == &Item::LAPIS_LAZULI {
                    if !self.insert_item(&mut slot_stack, 1, 2, false).await
                        && !self.insert_item(&mut slot_stack, 0, 1, false).await
                    {
                        return ItemStack::EMPTY.clone();
                    }
                } else if !self.insert_item(&mut slot_stack, 0, 1, false).await {
                    // Try within player inventory
                    if slot_index < 29 {
                        if !self.insert_item(&mut slot_stack, 29, 38, false).await {
                            return ItemStack::EMPTY.clone();
                        }
                    } else if !self.insert_item(&mut slot_stack, 2, 29, false).await {
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

/// Check if an item can be enchanted at an enchanting table.
/// Accepts books and any item with the `enchantable/durability` tag (tools, armor, weapons).
#[must_use]
pub fn can_enchant(item: &'static Item) -> bool {
    item == &Item::BOOK || item.has_tag(&tag::Item::MINECRAFT_ENCHANTABLE_DURABILITY)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enchanting_inventory_size() {
        let inv = EnchantingTableInventory::new();
        assert_eq!(inv.size(), 2);
    }

    #[test]
    fn lapis_slot_accepts_lapis() {
        let inv = Arc::new(EnchantingTableInventory::new());
        let slot = LapisSlot::new(inv, 1);
        let lapis = ItemStack::new(1, &Item::LAPIS_LAZULI);
        let rt = tokio::runtime::Runtime::new().unwrap();
        assert!(rt.block_on(slot.can_insert(&lapis)));
    }

    #[test]
    fn lapis_slot_rejects_non_lapis() {
        let inv = Arc::new(EnchantingTableInventory::new());
        let slot = LapisSlot::new(inv, 1);
        let diamond = ItemStack::new(1, &Item::DIAMOND);
        let rt = tokio::runtime::Runtime::new().unwrap();
        assert!(!rt.block_on(slot.can_insert(&diamond)));
    }

    #[test]
    fn book_is_enchantable() {
        assert!(can_enchant(&Item::BOOK));
    }

    #[test]
    fn diamond_sword_is_enchantable() {
        assert!(can_enchant(&Item::DIAMOND_SWORD));
    }
}
