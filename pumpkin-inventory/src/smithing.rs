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

/// A 3-slot inventory for the smithing table inputs (template, base, addition).
pub struct SmithingInventory {
    pub items: [Arc<Mutex<ItemStack>>; 3],
}

impl SmithingInventory {
    pub fn new() -> Self {
        Self {
            items: [
                Arc::new(Mutex::new(ItemStack::EMPTY.clone())),
                Arc::new(Mutex::new(ItemStack::EMPTY.clone())),
                Arc::new(Mutex::new(ItemStack::EMPTY.clone())),
            ],
        }
    }
}

impl Inventory for SmithingInventory {
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

impl Clearable for SmithingInventory {
    fn clear(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            for slot in &self.items {
                *slot.lock().await = ItemStack::EMPTY.clone();
            }
        })
    }
}

/// Output slot for the smithing table. Consumes one item from each input when taken.
pub struct SmithingOutputSlot {
    pub input_inventory: Arc<SmithingInventory>,
    pub id: AtomicU8,
    pub result: Arc<Mutex<ItemStack>>,
}

impl SmithingOutputSlot {
    pub fn new(input_inventory: Arc<SmithingInventory>) -> Self {
        Self {
            input_inventory,
            id: AtomicU8::new(0),
            result: Arc::new(Mutex::new(ItemStack::EMPTY.clone())),
        }
    }
}

impl Slot for SmithingOutputSlot {
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
            // Consume one item from each of the 3 input slots (template, base, addition)
            for i in 0..3 {
                let input_stack = self.input_inventory.get_stack(i).await;
                let mut guard = input_stack.lock().await;
                if !guard.is_empty() {
                    guard.item_count -= 1;
                }
            }
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

/// SmithingScreenHandler — vanilla SmithingMenu equivalent.
///
/// Layout:
/// - Slot 0: Template slot (smithing template item)
/// - Slot 1: Base slot (item to upgrade, e.g. diamond gear)
/// - Slot 2: Addition slot (upgrade material, e.g. netherite ingot)
/// - Slot 3: Output slot (result)
/// - Slots 4-30: Player inventory (3×9)
/// - Slots 31-39: Player hotbar (9)
///
/// Note: Actual recipe matching for smithing_transform and smithing_trim
/// requires smithing recipe data to be generated by pumpkin-data.
/// Currently the handler provides the correct slot structure and
/// quick-move logic. Recipe matching will be added once RECIPES_SMITHING
/// is available from the Architect.
pub struct SmithingScreenHandler {
    pub input_inventory: Arc<SmithingInventory>,
    pub output_slot: Arc<SmithingOutputSlot>,
    behaviour: ScreenHandlerBehaviour,
}

impl SmithingScreenHandler {
    pub async fn new(sync_id: u8, player_inventory: &Arc<PlayerInventory>) -> Self {
        let input_inventory = Arc::new(SmithingInventory::new());
        let output_slot = Arc::new(SmithingOutputSlot::new(input_inventory.clone()));

        let mut handler = Self {
            input_inventory: input_inventory.clone(),
            output_slot: output_slot.clone(),
            behaviour: ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Smithing)),
        };

        // Slot 0: Template
        handler.add_slot(Arc::new(NormalSlot::new(input_inventory.clone(), 0)));
        // Slot 1: Base
        handler.add_slot(Arc::new(NormalSlot::new(input_inventory.clone(), 1)));
        // Slot 2: Addition
        handler.add_slot(Arc::new(NormalSlot::new(input_inventory.clone(), 2)));
        // Slot 3: Output
        handler.add_slot(output_slot);

        // Player inventory + hotbar
        let player_inventory: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&player_inventory);

        handler
    }
}

impl ScreenHandler for SmithingScreenHandler {
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

            if slot_index == 3 {
                // From output slot — move to player inventory (4..40)
                if !self.insert_item(&mut slot_stack, 4, 40, true).await {
                    return ItemStack::EMPTY.clone();
                }
                slot.on_take_item(player, &stack_prev).await;
            } else if (0..3).contains(&slot_index) {
                // From input slots — move to player inventory (4..40)
                if !self.insert_item(&mut slot_stack, 4, 40, false).await {
                    return ItemStack::EMPTY.clone();
                }
            } else if (4..40).contains(&slot_index) {
                // From player inventory — try input slots (0..3)
                if !self.insert_item(&mut slot_stack, 0, 3, false).await {
                    // Try within player inventory
                    if slot_index < 31 {
                        if !self.insert_item(&mut slot_stack, 31, 40, false).await {
                            return ItemStack::EMPTY.clone();
                        }
                    } else if !self.insert_item(&mut slot_stack, 4, 31, false).await {
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
    fn smithing_inventory_size() {
        let inv = SmithingInventory::new();
        assert_eq!(inv.size(), 3);
    }

    #[tokio::test]
    async fn smithing_inventory_starts_empty() {
        let inv = SmithingInventory::new();
        assert!(inv.is_empty().await);
    }

    #[tokio::test]
    async fn smithing_inventory_set_all_slots() {
        let inv = SmithingInventory::new();

        // Slot 0: Template
        inv.set_stack(
            0,
            ItemStack::new(
                1,
                &pumpkin_data::item::Item::NETHERITE_UPGRADE_SMITHING_TEMPLATE,
            ),
        )
        .await;
        // Slot 1: Base
        inv.set_stack(1, ItemStack::new(1, &pumpkin_data::item::Item::DIAMOND_AXE))
            .await;
        // Slot 2: Addition
        inv.set_stack(
            2,
            ItemStack::new(1, &pumpkin_data::item::Item::NETHERITE_INGOT),
        )
        .await;

        assert!(!inv.is_empty().await);

        // Verify each slot
        let s0 = inv.get_stack(0).await;
        assert_eq!(s0.lock().await.item_count, 1);

        let s1 = inv.get_stack(1).await;
        assert_eq!(s1.lock().await.item_count, 1);

        let s2 = inv.get_stack(2).await;
        assert_eq!(s2.lock().await.item_count, 1);
    }

    #[tokio::test]
    async fn smithing_inventory_clear() {
        let inv = SmithingInventory::new();
        inv.set_stack(0, ItemStack::new(1, &pumpkin_data::item::Item::STONE))
            .await;
        inv.set_stack(1, ItemStack::new(1, &pumpkin_data::item::Item::DIRT))
            .await;
        inv.set_stack(2, ItemStack::new(1, &pumpkin_data::item::Item::STONE))
            .await;

        assert!(!inv.is_empty().await);
        inv.clear().await;
        assert!(inv.is_empty().await);
    }

    #[tokio::test]
    async fn smithing_inventory_remove_stack() {
        let inv = SmithingInventory::new();
        inv.set_stack(
            1,
            ItemStack::new(1, &pumpkin_data::item::Item::DIAMOND_SWORD),
        )
        .await;

        let removed = inv.remove_stack(1).await;
        assert_eq!(removed.item_count, 1);

        let stack = inv.get_stack(1).await;
        assert!(stack.lock().await.is_empty());
    }

    #[tokio::test]
    async fn smithing_output_slot_cannot_insert() {
        let inv = Arc::new(SmithingInventory::new());
        let output = SmithingOutputSlot::new(inv);
        let item = ItemStack::new(1, &pumpkin_data::item::Item::NETHERITE_AXE);
        assert!(!output.can_insert(&item).await);
    }

    #[tokio::test]
    async fn smithing_output_slot_starts_empty() {
        let inv = Arc::new(SmithingInventory::new());
        let output = SmithingOutputSlot::new(inv);
        assert!(!output.has_stack().await);
    }

    #[tokio::test]
    async fn smithing_output_slot_set_and_get() {
        let inv = Arc::new(SmithingInventory::new());
        let output = SmithingOutputSlot::new(inv);

        let item = ItemStack::new(1, &pumpkin_data::item::Item::NETHERITE_AXE);
        output.set_stack(item).await;

        assert!(output.has_stack().await);
        let stack = output.get_cloned_stack().await;
        assert_eq!(stack.item_count, 1);
    }
}
