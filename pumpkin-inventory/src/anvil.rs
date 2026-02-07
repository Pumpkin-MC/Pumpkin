//! Anvil screen handler — item repair, renaming, and enchantment combining.
//!
//! Vanilla slot layout (AnvilMenu.java):
//! - Slot 0: First input (left)
//! - Slot 1: Second input (right)
//! - Slot 2: Output (result)
//! - Slots 3-38: Player inventory (3-29 main, 30-38 hotbar)

use std::{any::Any, sync::Arc};

use pumpkin_data::data_component_impl::EnchantmentsImpl;
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

/// The anvil's internal 2-slot inventory (input left + input right).
pub struct AnvilInventory {
    slots: Vec<Arc<Mutex<ItemStack>>>,
    dirty: std::sync::atomic::AtomicBool,
}

impl AnvilInventory {
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

impl Inventory for AnvilInventory {
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

impl Clearable for AnvilInventory {
    fn clear(&self) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            for slot in &self.slots {
                *slot.lock().await = ItemStack::EMPTY.clone();
            }
        })
    }
}

/// Output slot for the anvil — cannot insert directly, only take.
pub struct AnvilOutputSlot {
    id: std::sync::atomic::AtomicU8,
    result: Arc<Mutex<ItemStack>>,
}

impl AnvilOutputSlot {
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: std::sync::atomic::AtomicU8::new(0),
            result: Arc::new(Mutex::new(ItemStack::EMPTY.clone())),
        }
    }
}

impl Slot for AnvilOutputSlot {
    fn get_inventory(&self) -> Arc<dyn Inventory> {
        // Output slot is standalone, return a dummy
        Arc::new(AnvilInventory::new())
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

/// Compute the anvil result from two input stacks.
///
/// Returns `(result, level_cost)` or `None` if no valid combination.
///
/// Vanilla anvil mechanics:
/// 1. Repair: two same-type damageable items → combined durability
/// 2. Enchant combine: item + enchanted book or same item → merged enchantments
/// 3. Material repair: item + repair material → restore durability
/// TODO: Renaming (requires text input packet from client)
/// TODO: RepairCost tracking (component is stub)
pub fn compute_anvil_result(
    left: &ItemStack,
    right: &ItemStack,
) -> Option<(ItemStack, i32)> {
    if left.is_empty() {
        return None;
    }

    // Case 1: Right slot is empty — rename only (needs text input, not yet supported)
    if right.is_empty() {
        return None;
    }

    let mut result = left.clone();
    let mut cost = 0i32;

    // Case 2: Same item type — repair + enchantment merge
    if left.item == right.item && left.is_damageable() {
        // Combine durability
        let max_damage = left.get_max_damage().unwrap_or(0);
        let left_remaining = max_damage - left.get_damage();
        let right_remaining = max_damage - right.get_damage();
        let bonus = max_damage * 12 / 100; // 12% bonus
        let new_remaining = (left_remaining + right_remaining + bonus).min(max_damage);
        result.set_damage((max_damage - new_remaining).max(0));
        cost += 2;

        // Merge enchantments from right into result
        cost += merge_enchantments(&mut result, right);

        return Some((result, cost.max(1)));
    }

    // Case 3: Right is an enchanted book — apply enchantments to left
    if right.item == &Item::ENCHANTED_BOOK {
        cost += merge_enchantments(&mut result, right);
        if cost > 0 {
            return Some((result, cost));
        }
        return None;
    }

    // Case 4: Material repair — check if right item is a valid repair material
    if left.is_damageable() && is_repair_material(left.item, right.item) {
        let max_damage = left.get_max_damage().unwrap_or(0);
        let repair_per_material = max_damage / 4; // 25% per material
        let repair_amount = repair_per_material * i32::from(right.item_count);
        let current_damage = left.get_damage();
        let new_damage = (current_damage - repair_amount).max(0);
        result.set_damage(new_damage);
        cost += i32::from(right.item_count);
        return Some((result, cost.max(1)));
    }

    None
}

/// Merge enchantments from `source` into `target`.
/// Returns the level cost for the merge.
fn merge_enchantments(target: &mut ItemStack, source: &ItemStack) -> i32 {
    let source_enchants = source.get_data_component::<EnchantmentsImpl>();
    let Some(source_data) = source_enchants else {
        return 0;
    };
    if source_data.enchantment.is_empty() {
        return 0;
    }

    let mut cost = 0i32;
    for &(enchantment, source_level) in source_data.enchantment.iter() {
        let current_level = target.get_enchantment_level(enchantment);
        let new_level = if current_level == source_level {
            // Same level: combine (e.g., Sharpness III + III = IV)
            (source_level + 1).min(enchantment.max_level)
        } else {
            // Different levels: take the higher
            current_level.max(source_level)
        };

        if new_level > current_level {
            target.enchant(enchantment, new_level);
            cost += new_level;
        }
    }

    cost
}

/// Check if `material` is a valid repair material for items of `item` type.
///
/// Repair tags contain the MATERIALS (e.g., `REPAIRS_DIAMOND_ARMOR` contains "diamond").
/// We check if the material is in any repair tag, then verify the item belongs to
/// the matching material category via registry key prefix.
///
/// TODO: When RepairableImpl is no longer a stub, query `item.repairable` directly.
fn is_repair_material(item: &'static Item, material: &'static Item) -> bool {
    let name = item
        .registry_key
        .strip_prefix("minecraft:")
        .unwrap_or(item.registry_key);

    (material.has_tag(&tag::Item::MINECRAFT_REPAIRS_DIAMOND_ARMOR) && name.starts_with("diamond_"))
        || (material.has_tag(&tag::Item::MINECRAFT_REPAIRS_IRON_ARMOR)
            && name.starts_with("iron_"))
        || (material.has_tag(&tag::Item::MINECRAFT_REPAIRS_GOLD_ARMOR)
            && name.starts_with("golden_"))
        || (material.has_tag(&tag::Item::MINECRAFT_REPAIRS_LEATHER_ARMOR)
            && name.starts_with("leather_"))
        || (material.has_tag(&tag::Item::MINECRAFT_REPAIRS_NETHERITE_ARMOR)
            && name.starts_with("netherite_"))
        || (material.has_tag(&tag::Item::MINECRAFT_REPAIRS_CHAIN_ARMOR)
            && name.starts_with("chainmail_"))
        || (material.has_tag(&tag::Item::MINECRAFT_REPAIRS_TURTLE_HELMET)
            && name == "turtle_helmet")
        || (material.has_tag(&tag::Item::MINECRAFT_REPAIRS_WOLF_ARMOR) && name == "wolf_armor")
        || (material.has_tag(&tag::Item::MINECRAFT_REPAIRS_COPPER_ARMOR)
            && name.starts_with("copper_"))
}

/// Anvil screen handler.
///
/// Slot layout:
/// - 0: Left input
/// - 1: Right input
/// - 2: Output
/// - 3-38: Player inventory
pub struct AnvilScreenHandler {
    behaviour: ScreenHandlerBehaviour,
    inventory: Arc<AnvilInventory>,
    output_slot: Arc<AnvilOutputSlot>,
    /// Level cost for the current operation (sent to client as property 0)
    pub level_cost: i32,
}

impl AnvilScreenHandler {
    pub async fn new(
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
    ) -> Self {
        let inventory = Arc::new(AnvilInventory::new());
        let output_slot = Arc::new(AnvilOutputSlot::new());

        let mut handler = Self {
            behaviour: ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Anvil)),
            inventory: inventory.clone(),
            output_slot: output_slot.clone(),
            level_cost: 0,
        };

        // Slot 0: Left input
        handler.add_slot(Arc::new(crate::slot::NormalSlot::new(
            inventory.clone(),
            0,
        )));
        // Slot 1: Right input
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
        let left_stack = self.inventory.get_stack(0).await;
        let left = left_stack.lock().await;
        let right_stack = self.inventory.get_stack(1).await;
        let right = right_stack.lock().await;

        if let Some((result, cost)) = compute_anvil_result(&left, &right) {
            *self.output_slot.result.lock().await = result;
            self.level_cost = cost;
        } else {
            *self.output_slot.result.lock().await = ItemStack::EMPTY.clone();
            self.level_cost = 0;
        }
    }
}

impl ScreenHandler for AnvilScreenHandler {
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
    use pumpkin_data::Enchantment;

    #[test]
    fn anvil_inventory_size() {
        let inv = AnvilInventory::new();
        assert_eq!(inv.size(), 2);
    }

    #[test]
    fn anvil_output_cannot_insert() {
        let slot = AnvilOutputSlot::new();
        let stack = ItemStack::new(1, &Item::DIAMOND);
        let rt = tokio::runtime::Runtime::new().unwrap();
        assert!(!rt.block_on(slot.can_insert(&stack)));
    }

    #[test]
    fn anvil_repair_same_type() {
        let mut left = ItemStack::new(1, &Item::DIAMOND_PICKAXE);
        left.set_damage(500);
        let mut right = ItemStack::new(1, &Item::DIAMOND_PICKAXE);
        right.set_damage(500);

        let result = compute_anvil_result(&left, &right);
        assert!(result.is_some(), "Same-type damaged items should combine");
        let (result, cost) = result.unwrap();
        assert!(result.item == &Item::DIAMOND_PICKAXE);
        assert!(result.get_damage() < 500, "Damage should be reduced");
        assert!(cost >= 1, "Should have a level cost");
    }

    #[test]
    fn anvil_different_items_no_repair() {
        let left = ItemStack::new(1, &Item::DIAMOND_PICKAXE);
        let right = ItemStack::new(1, &Item::DIAMOND_AXE);
        assert!(compute_anvil_result(&left, &right).is_none());
    }

    #[test]
    fn anvil_enchanted_book_applies() {
        let left = ItemStack::new(1, &Item::DIAMOND_SWORD);
        let mut right = ItemStack::new(1, &Item::ENCHANTED_BOOK);
        right.enchant(&Enchantment::SHARPNESS, 3);

        let result = compute_anvil_result(&left, &right);
        assert!(result.is_some(), "Enchanted book should apply to sword");
        let (result, cost) = result.unwrap();
        assert_eq!(result.get_enchantment_level(&Enchantment::SHARPNESS), 3);
        assert!(cost >= 3);
    }

    #[test]
    fn anvil_enchantment_combine_same_level() {
        let mut left = ItemStack::new(1, &Item::DIAMOND_SWORD);
        left.enchant(&Enchantment::SHARPNESS, 3);
        let mut right = ItemStack::new(1, &Item::DIAMOND_SWORD);
        right.enchant(&Enchantment::SHARPNESS, 3);

        let result = compute_anvil_result(&left, &right);
        assert!(result.is_some());
        let (result, _cost) = result.unwrap();
        // Sharpness III + III = IV (capped at max_level=5)
        assert_eq!(result.get_enchantment_level(&Enchantment::SHARPNESS), 4);
    }

    #[test]
    fn anvil_material_repair_diamond() {
        let mut left = ItemStack::new(1, &Item::DIAMOND_PICKAXE);
        left.set_damage(800);
        let right = ItemStack::new(2, &Item::DIAMOND);

        let result = compute_anvil_result(&left, &right);
        assert!(result.is_some(), "Diamond should repair diamond pickaxe");
        let (result, _cost) = result.unwrap();
        assert!(result.get_damage() < 800, "Damage should decrease");
    }

    #[test]
    fn anvil_empty_right_no_result() {
        let left = ItemStack::new(1, &Item::DIAMOND_SWORD);
        let right = ItemStack::EMPTY.clone();
        assert!(compute_anvil_result(&left, &right).is_none());
    }

    #[test]
    fn anvil_empty_left_no_result() {
        let left = ItemStack::EMPTY.clone();
        let right = ItemStack::new(1, &Item::DIAMOND);
        assert!(compute_anvil_result(&left, &right).is_none());
    }

    #[test]
    fn anvil_non_damageable_no_repair() {
        // Two diamonds can't be repaired
        let left = ItemStack::new(1, &Item::DIAMOND);
        let right = ItemStack::new(1, &Item::DIAMOND);
        assert!(compute_anvil_result(&left, &right).is_none());
    }

    #[test]
    fn anvil_enchanted_book_no_enchants_no_result() {
        let left = ItemStack::new(1, &Item::DIAMOND_SWORD);
        let right = ItemStack::new(1, &Item::ENCHANTED_BOOK);
        // Book with no enchantments
        assert!(compute_anvil_result(&left, &right).is_none());
    }
}
