use std::{any::Any, pin::Pin, sync::Arc};

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

use std::sync::atomic::{AtomicU8, Ordering};

/// A simple inventory backing the stonecutter's input slot.
pub struct StonecutterInventory {
    pub items: [Arc<Mutex<ItemStack>>; 1],
}

impl StonecutterInventory {
    pub fn new() -> Self {
        Self {
            items: [Arc::new(Mutex::new(ItemStack::EMPTY.clone()))],
        }
    }
}

impl Inventory for StonecutterInventory {
    fn size(&self) -> usize {
        1
    }

    fn is_empty(&self) -> InventoryFuture<'_, bool> {
        Box::pin(async move { self.items[0].lock().await.is_empty() })
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

impl Clearable for StonecutterInventory {
    fn clear(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            *self.items[0].lock().await = ItemStack::EMPTY.clone();
        })
    }
}

/// Output slot for the stonecutter. Consumes one item from input when taken.
pub struct StonecutterOutputSlot {
    pub input_inventory: Arc<StonecutterInventory>,
    pub id: AtomicU8,
    pub result: Arc<Mutex<ItemStack>>,
}

impl StonecutterOutputSlot {
    pub fn new(input_inventory: Arc<StonecutterInventory>) -> Self {
        Self {
            input_inventory,
            id: AtomicU8::new(0),
            result: Arc::new(Mutex::new(ItemStack::EMPTY.clone())),
        }
    }
}

impl Slot for StonecutterOutputSlot {
    fn get_inventory(&self) -> Arc<dyn Inventory> {
        self.input_inventory.clone()
    }

    fn get_index(&self) -> usize {
        999 // Output slot does not belong to the backing inventory
    }

    fn set_id(&self, id: usize) {
        self.id.store(id as u8, Ordering::Relaxed);
    }

    fn on_take_item<'a>(
        &'a self,
        _player: &'a dyn InventoryPlayer,
        _stack: &'a ItemStack,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            // Consume one item from the input slot
            let input_stack = self.input_inventory.get_stack(0).await;
            let mut guard = input_stack.lock().await;
            if !guard.is_empty() {
                guard.item_count -= 1;
            }
            drop(guard);
            self.mark_dirty().await;
        })
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

    fn set_stack_prev(&self, stack: ItemStack, _previous_stack: ItemStack) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            *self.result.lock().await = stack;
        })
    }

    fn mark_dirty(&self) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.input_inventory.mark_dirty();
        })
    }

    fn take_stack(&self, _amount: u8) -> BoxFuture<'_, ItemStack> {
        Box::pin(async move {
            if self.has_stack().await {
                let stack = self.result.lock().await;
                stack.clone()
            } else {
                ItemStack::EMPTY.clone()
            }
        })
    }
}

/// StonecutterScreenHandler — vanilla StonecutterMenu equivalent.
///
/// Layout:
/// - Slot 0: Input slot (1 item)
/// - Slot 1: Output slot (result of selected recipe)
/// - Slots 2-28: Player inventory (3×9)
/// - Slots 29-37: Player hotbar (9)
///
/// Note: Actual recipe selection requires stonecutting recipe data to be generated
/// by pumpkin-data. Currently the handler provides the correct slot structure and
/// quick-move logic. Recipe matching is a placeholder until RECIPES_STONECUTTING
/// is available from the Architect.
pub struct StonecutterScreenHandler {
    pub input_inventory: Arc<StonecutterInventory>,
    pub output_slot: Arc<StonecutterOutputSlot>,
    behaviour: ScreenHandlerBehaviour,
}

impl StonecutterScreenHandler {
    pub async fn new(sync_id: u8, player_inventory: &Arc<PlayerInventory>) -> Self {
        let input_inventory = Arc::new(StonecutterInventory::new());
        let output_slot = Arc::new(StonecutterOutputSlot::new(input_inventory.clone()));

        let mut handler = Self {
            input_inventory: input_inventory.clone(),
            output_slot: output_slot.clone(),
            behaviour: ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Stonecutter)),
        };

        // Slot 0: Input
        handler.add_slot(Arc::new(NormalSlot::new(input_inventory.clone(), 0)));
        // Slot 1: Output
        handler.add_slot(output_slot);

        // Player inventory + hotbar
        let player_inventory: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&player_inventory);

        handler
    }
}

impl ScreenHandler for StonecutterScreenHandler {
    fn on_closed<'a>(&'a mut self, player: &'a dyn InventoryPlayer) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            self.default_on_closed(player).await;
            self.drop_inventory(player, self.input_inventory.clone())
                .await;
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
        _player: &'a dyn InventoryPlayer,
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

            if slot_index == 1 {
                // From output slot — move to player inventory (2..38)
                if !self.insert_item(&mut slot_stack, 2, 38, true).await {
                    return ItemStack::EMPTY.clone();
                }
                slot.on_take_item(
                    // We need a player ref but quick_move signature doesn't give us one
                    // in a usable way here — on_take_item consumes from input
                    _player,
                    &stack_prev,
                )
                .await;
            } else if slot_index == 0 {
                // From input slot — move to player inventory (2..38)
                if !self.insert_item(&mut slot_stack, 2, 38, false).await {
                    return ItemStack::EMPTY.clone();
                }
            } else if (2..38).contains(&slot_index) {
                // From player inventory — try input slot first (0..1)
                if !self.insert_item(&mut slot_stack, 0, 1, false).await {
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

            stack_prev
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stonecutter_inventory_size() {
        let inv = StonecutterInventory::new();
        assert_eq!(inv.size(), 1);
    }

    #[tokio::test]
    async fn stonecutter_inventory_starts_empty() {
        let inv = StonecutterInventory::new();
        assert!(inv.is_empty().await);
    }

    #[tokio::test]
    async fn stonecutter_inventory_set_and_get() {
        let inv = StonecutterInventory::new();
        let item = ItemStack::new(5, &pumpkin_data::item::Item::STONE);
        inv.set_stack(0, item).await;

        let stack = inv.get_stack(0).await;
        assert_eq!(stack.lock().await.item_count, 5);
        assert!(!inv.is_empty().await);
    }

    #[tokio::test]
    async fn stonecutter_inventory_clear() {
        let inv = StonecutterInventory::new();
        inv.set_stack(0, ItemStack::new(1, &pumpkin_data::item::Item::STONE))
            .await;
        assert!(!inv.is_empty().await);
        inv.clear().await;
        assert!(inv.is_empty().await);
    }

    #[tokio::test]
    async fn stonecutter_inventory_remove_stack() {
        let inv = StonecutterInventory::new();
        inv.set_stack(0, ItemStack::new(3, &pumpkin_data::item::Item::STONE))
            .await;

        let removed = inv.remove_stack(0).await;
        assert_eq!(removed.item_count, 3);
        assert!(inv.is_empty().await);
    }

    #[tokio::test]
    async fn stonecutter_output_slot_cannot_insert() {
        let inv = Arc::new(StonecutterInventory::new());
        let output = StonecutterOutputSlot::new(inv);
        let item = ItemStack::new(1, &pumpkin_data::item::Item::STONE);
        assert!(!output.can_insert(&item).await);
    }

    #[tokio::test]
    async fn stonecutter_output_slot_starts_empty() {
        let inv = Arc::new(StonecutterInventory::new());
        let output = StonecutterOutputSlot::new(inv);
        assert!(!output.has_stack().await);
    }

    #[tokio::test]
    async fn stonecutter_output_slot_set_and_get() {
        let inv = Arc::new(StonecutterInventory::new());
        let output = StonecutterOutputSlot::new(inv);

        let item = ItemStack::new(2, &pumpkin_data::item::Item::STONE_SLAB);
        output.set_stack(item).await;

        assert!(output.has_stack().await);
        let stack = output.get_cloned_stack().await;
        assert_eq!(stack.item_count, 2);
    }
}
