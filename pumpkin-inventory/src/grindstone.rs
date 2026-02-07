//! Grindstone screen handler — enchantment removal and item repair.
//!
//! Vanilla slot layout (GrindstoneMenu.java):
//! - Slot 0: Top input
//! - Slot 1: Bottom input
//! - Slot 2: Output (result)
//! - Slots 3-38: Player inventory (3-29 main, 30-38 hotbar)

use std::{any::Any, sync::Arc};

use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{DataComponentImpl, EnchantmentsImpl};
use pumpkin_data::screen::WindowType;
use pumpkin_world::inventory::{Clearable, Inventory, InventoryFuture, split_stack};
use pumpkin_world::item::ItemStack;
use tokio::sync::Mutex;

use crate::player::player_inventory::PlayerInventory;
use crate::screen_handler::{
    InventoryPlayer, ItemStackFuture, ScreenHandler, ScreenHandlerBehaviour, ScreenHandlerFuture,
};
use crate::slot::{BoxFuture, Slot};

/// The grindstone's 2-slot internal inventory (top + bottom input).
pub struct GrindstoneInventory {
    slots: Vec<Arc<Mutex<ItemStack>>>,
    dirty: std::sync::atomic::AtomicBool,
}

impl GrindstoneInventory {
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

impl Inventory for GrindstoneInventory {
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

impl Clearable for GrindstoneInventory {
    fn clear(&self) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            for slot in &self.slots {
                *slot.lock().await = ItemStack::EMPTY.clone();
            }
        })
    }
}

/// Output slot for the grindstone.
pub struct GrindstoneOutputSlot {
    id: std::sync::atomic::AtomicU8,
    result: Arc<Mutex<ItemStack>>,
}

impl GrindstoneOutputSlot {
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: std::sync::atomic::AtomicU8::new(0),
            result: Arc::new(Mutex::new(ItemStack::EMPTY.clone())),
        }
    }
}

impl Slot for GrindstoneOutputSlot {
    fn get_inventory(&self) -> Arc<dyn Inventory> {
        Arc::new(GrindstoneInventory::new())
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

/// Compute the grindstone result from one or two input stacks.
///
/// Grindstone mechanics (vanilla):
/// 1. Single enchanted item → same item with enchantments removed (curses kept)
/// 2. Two same-type items → combined durability, enchantments removed (curses kept)
/// 3. Returns XP proportional to removed enchantment levels (not computed here)
pub fn compute_grindstone_result(
    top: &ItemStack,
    bottom: &ItemStack,
) -> Option<ItemStack> {
    if top.is_empty() && bottom.is_empty() {
        return None;
    }

    // Case 1: Single item — remove enchantments
    if !top.is_empty() && bottom.is_empty() {
        return strip_enchantments(top);
    }

    if top.is_empty() && !bottom.is_empty() {
        return strip_enchantments(bottom);
    }

    // Case 2: Two items of same type — combine durability + remove enchantments
    if top.item != bottom.item {
        return None;
    }

    if !top.is_damageable() {
        return None;
    }

    let mut result = ItemStack::new(1, top.item);
    let max_damage = top.get_max_damage().unwrap_or(0);
    let top_remaining = max_damage - top.get_damage();
    let bottom_remaining = max_damage - bottom.get_damage();
    let bonus = max_damage * 5 / 100; // 5% bonus
    let new_remaining = (top_remaining + bottom_remaining + bonus).min(max_damage);
    result.set_damage((max_damage - new_remaining).max(0));

    // Remove all non-curse enchantments
    transfer_curses_only(&mut result, top);
    transfer_curses_only(&mut result, bottom);

    Some(result)
}

/// Strip non-curse enchantments from an item. Returns None if item has no enchantments.
fn strip_enchantments(stack: &ItemStack) -> Option<ItemStack> {
    let mut result = stack.clone();

    // Remove the enchantments component entirely
    result.patch.retain(|(id, _)| *id != DataComponent::Enchantments);

    // Re-add only curses if present
    if let Some(enchants) = stack.get_data_component::<EnchantmentsImpl>() {
        let curses: Vec<_> = enchants
            .enchantment
            .iter()
            .filter(|(e, _)| is_curse(e))
            .copied()
            .collect();

        if !curses.is_empty() {
            result.patch.push((
                DataComponent::Enchantments,
                Some(
                    EnchantmentsImpl {
                        enchantment: std::borrow::Cow::Owned(curses),
                    }
                    .to_dyn(),
                ),
            ));
        }

        // Only produce a result if we actually removed something
        let original_count = enchants.enchantment.len();
        let curse_count = enchants
            .enchantment
            .iter()
            .filter(|(e, _)| is_curse(e))
            .count();
        if original_count > curse_count {
            return Some(result);
        }
    }

    // If no enchantments were removed, still allow as passthrough for damageable items
    if stack.is_damageable() {
        return Some(result);
    }

    None
}

/// Transfer only curse enchantments from source to target.
fn transfer_curses_only(target: &mut ItemStack, source: &ItemStack) {
    if let Some(enchants) = source.get_data_component::<EnchantmentsImpl>() {
        for &(enchantment, level) in enchants.enchantment.iter() {
            if is_curse(enchantment) {
                target.enchant(enchantment, level);
            }
        }
    }
}

/// Check if an enchantment is a curse (binding or vanishing).
fn is_curse(enchantment: &pumpkin_data::Enchantment) -> bool {
    use pumpkin_data::Enchantment;
    enchantment == &Enchantment::BINDING_CURSE || enchantment == &Enchantment::VANISHING_CURSE
}

/// Grindstone screen handler.
pub struct GrindstoneScreenHandler {
    behaviour: ScreenHandlerBehaviour,
    inventory: Arc<GrindstoneInventory>,
    output_slot: Arc<GrindstoneOutputSlot>,
}

impl GrindstoneScreenHandler {
    pub async fn new(
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
    ) -> Self {
        let inventory = Arc::new(GrindstoneInventory::new());
        let output_slot = Arc::new(GrindstoneOutputSlot::new());

        let mut handler = Self {
            behaviour: ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Grindstone)),
            inventory: inventory.clone(),
            output_slot: output_slot.clone(),
        };

        // Slot 0: Top input
        handler.add_slot(Arc::new(crate::slot::NormalSlot::new(
            inventory.clone(),
            0,
        )));
        // Slot 1: Bottom input
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

    /// Recalculate the grindstone output based on current inputs.
    pub async fn update_result(&mut self) {
        let top_stack = self.inventory.get_stack(0).await;
        let top = top_stack.lock().await;
        let bottom_stack = self.inventory.get_stack(1).await;
        let bottom = bottom_stack.lock().await;

        if let Some(result) = compute_grindstone_result(&top, &bottom) {
            *self.output_slot.result.lock().await = result;
        } else {
            *self.output_slot.result.lock().await = ItemStack::EMPTY.clone();
        }
    }
}

impl ScreenHandler for GrindstoneScreenHandler {
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
                // Player inventory → input slots
                if !self.insert_item(&mut slot_stack, 0, 2, false).await {
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
    use pumpkin_data::Enchantment;
    use pumpkin_data::item::Item;

    #[test]
    fn grindstone_inventory_size() {
        let inv = GrindstoneInventory::new();
        assert_eq!(inv.size(), 2);
    }

    #[test]
    fn grindstone_output_cannot_insert() {
        let slot = GrindstoneOutputSlot::new();
        let stack = ItemStack::new(1, &Item::DIAMOND);
        let rt = tokio::runtime::Runtime::new().unwrap();
        assert!(!rt.block_on(slot.can_insert(&stack)));
    }

    #[test]
    fn grindstone_strip_enchantment() {
        let mut sword = ItemStack::new(1, &Item::DIAMOND_SWORD);
        sword.enchant(&Enchantment::SHARPNESS, 5);
        let empty = ItemStack::EMPTY.clone();

        let result = compute_grindstone_result(&sword, &empty);
        assert!(result.is_some(), "Enchanted sword should produce result");
        let result = result.unwrap();
        assert_eq!(
            result.get_enchantment_level(&Enchantment::SHARPNESS),
            0,
            "Sharpness should be removed"
        );
    }

    #[test]
    fn grindstone_keep_curse() {
        let mut sword = ItemStack::new(1, &Item::DIAMOND_SWORD);
        sword.enchant(&Enchantment::SHARPNESS, 3);
        sword.enchant(&Enchantment::BINDING_CURSE, 1);
        let empty = ItemStack::EMPTY.clone();

        let result = compute_grindstone_result(&sword, &empty);
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(
            result.get_enchantment_level(&Enchantment::SHARPNESS),
            0,
            "Sharpness removed"
        );
        assert_eq!(
            result.get_enchantment_level(&Enchantment::BINDING_CURSE),
            1,
            "Curse kept"
        );
    }

    #[test]
    fn grindstone_combine_durability() {
        let mut top = ItemStack::new(1, &Item::DIAMOND_SWORD);
        top.set_damage(800);
        let mut bottom = ItemStack::new(1, &Item::DIAMOND_SWORD);
        bottom.set_damage(800);

        let result = compute_grindstone_result(&top, &bottom);
        assert!(result.is_some(), "Two damaged swords should combine");
        let result = result.unwrap();
        assert!(result.get_damage() < 800, "Combined damage should be less");
    }

    #[test]
    fn grindstone_different_items_no_combine() {
        let top = ItemStack::new(1, &Item::DIAMOND_SWORD);
        let bottom = ItemStack::new(1, &Item::DIAMOND_AXE);
        assert!(
            compute_grindstone_result(&top, &bottom).is_none(),
            "Different items can't combine"
        );
    }

    #[test]
    fn grindstone_both_empty_no_result() {
        let empty = ItemStack::EMPTY.clone();
        assert!(compute_grindstone_result(&empty, &empty).is_none());
    }

    #[test]
    fn grindstone_non_enchanted_damageable_passthrough() {
        // A plain damaged sword should still produce a result (passthrough)
        let mut sword = ItemStack::new(1, &Item::DIAMOND_SWORD);
        sword.set_damage(100);
        let empty = ItemStack::EMPTY.clone();

        let result = compute_grindstone_result(&sword, &empty);
        assert!(
            result.is_some(),
            "Damageable item should passthrough grindstone"
        );
    }
}
