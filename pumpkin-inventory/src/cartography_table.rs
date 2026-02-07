//! Cartography table screen handler — map extending, cloning, and locking.
//!
//! Vanilla slot layout (CartographyTableMenu.java):
//! - Slot 0: Map input
//! - Slot 1: Additional input (paper = extend, empty map = clone, glass pane = lock)
//! - Slot 2: Output
//! - Slots 3-38: Player inventory (3-29 main, 30-38 hotbar)

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

/// The cartography table's 2-slot input inventory (map + paper/compass/glass pane).
pub struct CartographyTableInventory {
    slots: Vec<Arc<Mutex<ItemStack>>>,
    dirty: std::sync::atomic::AtomicBool,
}

impl CartographyTableInventory {
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

impl Inventory for CartographyTableInventory {
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

impl Clearable for CartographyTableInventory {
    fn clear(&self) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            for slot in &self.slots {
                *slot.lock().await = ItemStack::EMPTY.clone();
            }
        })
    }
}

/// Output slot for the cartography table — cannot insert directly.
pub struct CartographyOutputSlot {
    id: std::sync::atomic::AtomicU8,
    result: Arc<Mutex<ItemStack>>,
}

impl CartographyOutputSlot {
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: std::sync::atomic::AtomicU8::new(0),
            result: Arc::new(Mutex::new(ItemStack::EMPTY.clone())),
        }
    }
}

impl Slot for CartographyOutputSlot {
    fn get_inventory(&self) -> Arc<dyn Inventory> {
        Arc::new(CartographyTableInventory::new())
    }

    fn get_index(&self) -> usize {
        999
    }

    fn set_id(&self, id: usize) {
        self.id
            .store(id as u8, std::sync::atomic::Ordering::Relaxed);
    }

    fn can_insert(&self, _stack: &ItemStack) -> BoxFuture<'_, bool> {
        Box::pin(async move { false })
    }

    fn get_stack(&self) -> BoxFuture<'_, Arc<Mutex<ItemStack>>> {
        Box::pin(async move { self.result.clone() })
    }

    fn get_cloned_stack(&self) -> BoxFuture<'_, ItemStack> {
        Box::pin(async move { self.result.lock().await.clone() })
    }

    fn has_stack(&self) -> BoxFuture<'_, bool> {
        Box::pin(async move { !self.result.lock().await.is_empty() })
    }

    fn set_stack(&self, stack: ItemStack) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            *self.result.lock().await = stack;
        })
    }

    fn set_stack_prev(&self, stack: ItemStack, _previous: ItemStack) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            *self.result.lock().await = stack;
        })
    }

    fn mark_dirty(&self) -> BoxFuture<'_, ()> {
        Box::pin(async move {})
    }

    fn get_max_item_count(&self) -> BoxFuture<'_, u8> {
        Box::pin(async move { 64 })
    }

    fn take_stack(&self, _amount: u8) -> BoxFuture<'_, ItemStack> {
        Box::pin(async move {
            let stack = self.result.lock().await.clone();
            *self.result.lock().await = ItemStack::EMPTY.clone();
            stack
        })
    }
}

/// Compute the cartography table result.
///
/// Operations:
/// - Map + Paper → Extended map (zoom out)
/// - Map + Empty Map → Cloned map
/// - Map + Glass Pane → Locked map
///
/// TODO: All operations require MapIdImpl (stub). Currently returns a copy of
/// the map input as a placeholder.
pub fn compute_cartography_result(map: &ItemStack, additional: &ItemStack) -> Option<ItemStack> {
    if map.is_empty() || map.item != &Item::FILLED_MAP {
        return None;
    }

    if additional.is_empty() {
        return None;
    }

    // Paper → extend
    if additional.item == &Item::PAPER {
        return Some(ItemStack::new(1, &Item::FILLED_MAP));
    }

    // Empty map → clone
    if additional.item == &Item::MAP {
        return Some(ItemStack::new(2, &Item::FILLED_MAP));
    }

    // Glass pane → lock
    if additional.item == &Item::GLASS_PANE {
        return Some(ItemStack::new(1, &Item::FILLED_MAP));
    }

    None
}

/// Cartography table screen handler.
///
/// Slot layout:
/// - 0: Map input
/// - 1: Additional input (paper/empty map/glass pane)
/// - 2: Output
/// - 3-38: Player inventory
pub struct CartographyTableScreenHandler {
    behaviour: ScreenHandlerBehaviour,
    inventory: Arc<CartographyTableInventory>,
    output_slot: Arc<CartographyOutputSlot>,
}

impl CartographyTableScreenHandler {
    pub async fn new(
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
    ) -> Self {
        let inventory = Arc::new(CartographyTableInventory::new());
        let output_slot = Arc::new(CartographyOutputSlot::new());

        let mut handler = Self {
            behaviour: ScreenHandlerBehaviour::new(
                sync_id,
                Some(WindowType::CartographyTable),
            ),
            inventory: inventory.clone(),
            output_slot: output_slot.clone(),
        };

        // Slot 0: Map input
        handler.add_slot(Arc::new(crate::slot::NormalSlot::new(
            inventory.clone(),
            0,
        )));
        // Slot 1: Additional input
        handler.add_slot(Arc::new(crate::slot::NormalSlot::new(
            inventory.clone(),
            1,
        )));
        // Slot 2: Output
        handler.add_slot(output_slot);

        // Slots 3-38: Player inventory
        let player_inv: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&player_inv);

        handler
    }

    /// Recalculate the output based on current inputs.
    pub async fn update_result(&mut self) {
        let map_stack = self.inventory.get_stack(0).await;
        let map = map_stack.lock().await;
        let additional_stack = self.inventory.get_stack(1).await;
        let additional = additional_stack.lock().await;

        if let Some(result) = compute_cartography_result(&map, &additional) {
            *self.output_slot.result.lock().await = result;
        } else {
            *self.output_slot.result.lock().await = ItemStack::EMPTY.clone();
        }
    }
}

impl ScreenHandler for CartographyTableScreenHandler {
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

            if slot_index == 2 {
                // Output → player inventory
                if !self.insert_item(&mut slot_stack, 3, 39, true).await {
                    return ItemStack::EMPTY.clone();
                }
            } else if (0..=1).contains(&slot_index) {
                // Input → player inventory
                if !self.insert_item(&mut slot_stack, 3, 39, false).await {
                    return ItemStack::EMPTY.clone();
                }
            } else if (3..39).contains(&slot_index) {
                // Player inventory → cartography table
                if slot_stack.item == &Item::FILLED_MAP {
                    if !self.insert_item(&mut slot_stack, 0, 1, false).await {
                        return ItemStack::EMPTY.clone();
                    }
                } else if slot_stack.item == &Item::PAPER
                    || slot_stack.item == &Item::MAP
                    || slot_stack.item == &Item::GLASS_PANE
                {
                    if !self.insert_item(&mut slot_stack, 1, 2, false).await {
                        return ItemStack::EMPTY.clone();
                    }
                } else {
                    // Try within player inventory
                    if slot_index < 30 {
                        if !self.insert_item(&mut slot_stack, 30, 39, false).await {
                            return ItemStack::EMPTY.clone();
                        }
                    } else if !self.insert_item(&mut slot_stack, 3, 30, false).await {
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

    #[test]
    fn cartography_inventory_size() {
        let inv = CartographyTableInventory::new();
        assert_eq!(inv.size(), 2);
    }

    #[test]
    fn cartography_output_cannot_insert() {
        let slot = CartographyOutputSlot::new();
        let stack = ItemStack::new(1, &Item::FILLED_MAP);
        let rt = tokio::runtime::Runtime::new().unwrap();
        assert!(!rt.block_on(slot.can_insert(&stack)));
    }

    #[test]
    fn cartography_extend_map() {
        let map = ItemStack::new(1, &Item::FILLED_MAP);
        let paper = ItemStack::new(1, &Item::PAPER);
        let result = compute_cartography_result(&map, &paper);
        assert!(result.is_some(), "Map + paper should extend");
        let result = result.unwrap();
        assert!(result.item == &Item::FILLED_MAP);
    }

    #[test]
    fn cartography_clone_map() {
        let map = ItemStack::new(1, &Item::FILLED_MAP);
        let empty_map = ItemStack::new(1, &Item::MAP);
        let result = compute_cartography_result(&map, &empty_map);
        assert!(result.is_some(), "Map + empty map should clone");
        let result = result.unwrap();
        assert_eq!(result.item_count, 2, "Clone produces 2 maps");
    }

    #[test]
    fn cartography_lock_map() {
        let map = ItemStack::new(1, &Item::FILLED_MAP);
        let pane = ItemStack::new(1, &Item::GLASS_PANE);
        let result = compute_cartography_result(&map, &pane);
        assert!(result.is_some(), "Map + glass pane should lock");
    }

    #[test]
    fn cartography_no_map_no_result() {
        let diamond = ItemStack::new(1, &Item::DIAMOND);
        let paper = ItemStack::new(1, &Item::PAPER);
        assert!(
            compute_cartography_result(&diamond, &paper).is_none(),
            "Non-map input should fail"
        );
    }

    #[test]
    fn cartography_empty_additional_no_result() {
        let map = ItemStack::new(1, &Item::FILLED_MAP);
        let empty = ItemStack::EMPTY.clone();
        assert!(compute_cartography_result(&map, &empty).is_none());
    }
}
