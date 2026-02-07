//! Loom screen handler — banner pattern application.
//!
//! Vanilla slot layout (LoomMenu.java):
//! - Slot 0: Banner input
//! - Slot 1: Dye input
//! - Slot 2: Pattern item (optional, for special patterns)
//! - Slot 3: Output (banner with applied pattern)
//! - Slots 4-39: Player inventory (4-30 main, 31-39 hotbar)

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

/// The loom's 3-slot input inventory (banner + dye + pattern item).
pub struct LoomInventory {
    slots: Vec<Arc<Mutex<ItemStack>>>,
    dirty: std::sync::atomic::AtomicBool,
}

impl LoomInventory {
    #[must_use]
    pub fn new() -> Self {
        Self {
            slots: vec![
                Arc::new(Mutex::new(ItemStack::EMPTY.clone())),
                Arc::new(Mutex::new(ItemStack::EMPTY.clone())),
                Arc::new(Mutex::new(ItemStack::EMPTY.clone())),
            ],
            dirty: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

impl Inventory for LoomInventory {
    fn size(&self) -> usize {
        3
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

impl Clearable for LoomInventory {
    fn clear(&self) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            for slot in &self.slots {
                *slot.lock().await = ItemStack::EMPTY.clone();
            }
        })
    }
}

/// Output slot for the loom — cannot insert directly.
pub struct LoomOutputSlot {
    id: std::sync::atomic::AtomicU8,
    result: Arc<Mutex<ItemStack>>,
}

impl LoomOutputSlot {
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: std::sync::atomic::AtomicU8::new(0),
            result: Arc::new(Mutex::new(ItemStack::EMPTY.clone())),
        }
    }
}

impl Slot for LoomOutputSlot {
    fn get_inventory(&self) -> Arc<dyn Inventory> {
        Arc::new(LoomInventory::new())
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

/// Check if an item is a banner (any color).
pub fn is_banner(item: &'static Item) -> bool {
    item.has_tag(&tag::Item::MINECRAFT_BANNERS)
}

/// Check if an item is a dye.
pub fn is_dye(item: &'static Item) -> bool {
    item.has_tag(&tag::Item::MINECRAFT_DYEABLE)
        || item == &Item::WHITE_DYE
        || item == &Item::ORANGE_DYE
        || item == &Item::MAGENTA_DYE
        || item == &Item::LIGHT_BLUE_DYE
        || item == &Item::YELLOW_DYE
        || item == &Item::LIME_DYE
        || item == &Item::PINK_DYE
        || item == &Item::GRAY_DYE
        || item == &Item::LIGHT_GRAY_DYE
        || item == &Item::CYAN_DYE
        || item == &Item::PURPLE_DYE
        || item == &Item::BLUE_DYE
        || item == &Item::BROWN_DYE
        || item == &Item::GREEN_DYE
        || item == &Item::RED_DYE
        || item == &Item::BLACK_DYE
}

/// Check if an item is a banner pattern item (special patterns like creeper, skull, etc.).
pub fn is_banner_pattern_item(item: &'static Item) -> bool {
    item == &Item::CREEPER_BANNER_PATTERN
        || item == &Item::SKULL_BANNER_PATTERN
        || item == &Item::FLOWER_BANNER_PATTERN
        || item == &Item::MOJANG_BANNER_PATTERN
        || item == &Item::GLOBE_BANNER_PATTERN
        || item == &Item::PIGLIN_BANNER_PATTERN
        || item == &Item::FLOW_BANNER_PATTERN
        || item == &Item::GUSTER_BANNER_PATTERN
        || item == &Item::FIELD_MASONED_BANNER_PATTERN
        || item == &Item::BORDURE_INDENTED_BANNER_PATTERN
}

/// Loom screen handler.
///
/// Slot layout:
/// - 0: Banner input
/// - 1: Dye input
/// - 2: Pattern item (optional)
/// - 3: Output
/// - 4-39: Player inventory
///
/// Window properties:
/// - 0: Selected pattern index
///
/// TODO: Pattern application requires BannerPatternsImpl (currently stub).
/// The output banner should be a copy of the input with the selected pattern
/// applied as a new layer using the dye color.
pub struct LoomScreenHandler {
    behaviour: ScreenHandlerBehaviour,
    inventory: Arc<LoomInventory>,
    output_slot: Arc<LoomOutputSlot>,
    /// Currently selected pattern index (sent as window property 0)
    pub selected_pattern: i32,
}

impl LoomScreenHandler {
    pub async fn new(
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
    ) -> Self {
        let inventory = Arc::new(LoomInventory::new());
        let output_slot = Arc::new(LoomOutputSlot::new());

        let mut handler = Self {
            behaviour: ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Loom)),
            inventory: inventory.clone(),
            output_slot: output_slot.clone(),
            selected_pattern: -1,
        };

        // Slot 0: Banner input
        handler.add_slot(Arc::new(crate::slot::NormalSlot::new(
            inventory.clone(),
            0,
        )));
        // Slot 1: Dye input
        handler.add_slot(Arc::new(crate::slot::NormalSlot::new(
            inventory.clone(),
            1,
        )));
        // Slot 2: Pattern item
        handler.add_slot(Arc::new(crate::slot::NormalSlot::new(
            inventory.clone(),
            2,
        )));
        // Slot 3: Output
        handler.add_slot(output_slot);

        // Slots 4-39: Player inventory
        let player_inv: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&player_inv);

        handler
    }

    /// Recalculate the output based on current inputs and selected pattern.
    /// TODO: Implement pattern application when BannerPatternsImpl is available.
    pub async fn update_result(&mut self) {
        let banner_stack = self.inventory.get_stack(0).await;
        let banner = banner_stack.lock().await;
        let dye_stack = self.inventory.get_stack(1).await;
        let dye = dye_stack.lock().await;

        if banner.is_empty() || dye.is_empty() || self.selected_pattern < 0 {
            *self.output_slot.result.lock().await = ItemStack::EMPTY.clone();
            return;
        }

        // Produce a copy of the banner (pattern application requires BannerPatternsImpl)
        let result = ItemStack::new(1, banner.item);
        *self.output_slot.result.lock().await = result;
    }
}

impl ScreenHandler for LoomScreenHandler {
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

            if slot_index == 3 {
                // Output → player inventory
                if !self.insert_item(&mut slot_stack, 4, 40, true).await {
                    return ItemStack::EMPTY.clone();
                }
            } else if (0..=2).contains(&slot_index) {
                // Input slots → player inventory
                if !self.insert_item(&mut slot_stack, 4, 40, false).await {
                    return ItemStack::EMPTY.clone();
                }
            } else if (4..40).contains(&slot_index) {
                // Player inventory → loom
                if is_banner(slot_stack.item) {
                    if !self.insert_item(&mut slot_stack, 0, 1, false).await {
                        return ItemStack::EMPTY.clone();
                    }
                } else if is_banner_pattern_item(slot_stack.item) {
                    if !self.insert_item(&mut slot_stack, 2, 3, false).await {
                        return ItemStack::EMPTY.clone();
                    }
                } else if is_dye(slot_stack.item) {
                    if !self.insert_item(&mut slot_stack, 1, 2, false).await {
                        return ItemStack::EMPTY.clone();
                    }
                } else {
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

            slot.on_take_item(player, &stack).await;
            stack_prev
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loom_inventory_size() {
        let inv = LoomInventory::new();
        assert_eq!(inv.size(), 3);
    }

    #[test]
    fn loom_output_cannot_insert() {
        let slot = LoomOutputSlot::new();
        let stack = ItemStack::new(1, &Item::WHITE_BANNER);
        let rt = tokio::runtime::Runtime::new().unwrap();
        assert!(!rt.block_on(slot.can_insert(&stack)));
    }

    #[test]
    fn banner_pattern_items_detected() {
        assert!(is_banner_pattern_item(&Item::CREEPER_BANNER_PATTERN));
        assert!(is_banner_pattern_item(&Item::SKULL_BANNER_PATTERN));
        assert!(is_banner_pattern_item(&Item::GLOBE_BANNER_PATTERN));
        assert!(!is_banner_pattern_item(&Item::DIAMOND));
    }

    #[test]
    fn banners_detected() {
        assert!(is_banner(&Item::WHITE_BANNER));
        assert!(is_banner(&Item::RED_BANNER));
        assert!(!is_banner(&Item::DIAMOND));
    }
}
