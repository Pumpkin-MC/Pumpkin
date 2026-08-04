//! Loom menu shell: banner/dye/pattern slots and correct shift-click routing.
//!
//! Mirrors `net/minecraft/world/inventory/LoomMenu.java` (26.2 decompile) for slot layout
//! (`INV_SLOT_START`/`END` = 4/31, `USE_ROW_SLOT_START`/`END` = 31/40, banner/dye/pattern/result
//! at 0/1/2/3) and `quickMoveStack` (LoomMenu.java:211-261).
//!
//! The result slot's actual computation (`setupResultSlot`, LoomMenu.java:269-288) is not
//! implemented: it needs three data components/registry pieces Pumpkin does not have at all —
//! a per-item dye color value (`DataComponents.DYE`; `DyeImpl` is a unit struct here), a banner
//! pattern layer list on the banner `ItemStack` (`DataComponents.BANNER_PATTERNS`; Pumpkin only
//! has raw NBT patterns on the *block entity*, not a typed item component), and the banner
//! pattern registry itself (`DataComponents.PROVIDES_BANNER_PATTERNS` plus the vanilla pattern
//! list). Building those is a `pumpkin-data` change of its own, not a block-behavior diff.
//! Consequently the result slot here always stays empty and [`ScreenHandler::on_button_click`]
//! (pattern selection) always returns `false`. This still gives a working Loom: it opens, holds
//! the three input items with correct slot restrictions, and returns them on close.

use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::AtomicU8;

use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::screen::WindowType;
use pumpkin_data::tag::{Item as ItemTag, Taggable};
use pumpkin_world::inventory::Inventory;
use pumpkin_world::inventory::SimpleInventory;

use crate::player::player_inventory::PlayerInventory;
use crate::screen_handler::{
    InventoryPlayer, ItemStackFuture, ScreenHandler, ScreenHandlerBehaviour, ScreenHandlerFuture,
    offer_or_drop_stack,
};
use crate::slot::{BoxFuture, Slot};

fn is_banner(stack: &ItemStack) -> bool {
    stack.item.has_tag(&ItemTag::MINECRAFT_BANNERS)
}

fn is_dye_item(stack: &ItemStack) -> bool {
    stack.item.has_tag(&ItemTag::MINECRAFT_LOOM_DYES)
}

fn is_pattern_item(stack: &ItemStack) -> bool {
    stack.item.has_tag(&ItemTag::MINECRAFT_LOOM_PATTERNS)
}

pub struct LoomScreenHandler {
    behaviour: ScreenHandlerBehaviour,
    pub input_inventory: Arc<SimpleInventory>,
    pub output_inventory: Arc<SimpleInventory>,
}

impl LoomScreenHandler {
    pub fn new(sync_id: u8, player_inventory: &Arc<PlayerInventory>) -> Self {
        let behaviour = ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Loom));
        let input_inventory = Arc::new(SimpleInventory::new(3));
        let output_inventory = Arc::new(SimpleInventory::new(1));

        let mut handler = Self {
            behaviour,
            input_inventory: input_inventory.clone(),
            output_inventory: output_inventory.clone(),
        };

        handler.add_slot(Arc::new(LoomInputSlot::new(
            input_inventory.clone() as Arc<dyn Inventory>,
            0,
            is_banner,
        )));
        handler.add_slot(Arc::new(LoomInputSlot::new(
            input_inventory.clone() as Arc<dyn Inventory>,
            1,
            is_dye_item,
        )));
        handler.add_slot(Arc::new(LoomInputSlot::new(
            input_inventory as Arc<dyn Inventory>,
            2,
            is_pattern_item,
        )));
        handler.add_slot(Arc::new(LoomResultSlot::new(
            output_inventory as Arc<dyn Inventory>,
            0,
        )));

        let player_inventory: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&player_inventory);

        handler
    }
}

impl ScreenHandler for LoomScreenHandler {
    fn get_behaviour(&self) -> &ScreenHandlerBehaviour {
        &self.behaviour
    }

    fn get_behaviour_mut(&mut self) -> &mut ScreenHandlerBehaviour {
        &mut self.behaviour
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn on_closed<'a>(&'a mut self, player: &'a dyn InventoryPlayer) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            self.default_on_closed(player).await;
            for i in 0..3 {
                let stack = self.input_inventory.remove_stack(i).await;
                if !stack.is_empty() {
                    offer_or_drop_stack(player, stack).await;
                }
            }
        })
    }

    fn quick_move<'a>(
        &'a mut self,
        player: &'a dyn InventoryPlayer,
        slot_index: i32,
    ) -> ItemStackFuture<'a> {
        Box::pin(async move {
            let mut stack_left = ItemStack::EMPTY.clone();
            let slot = self.get_behaviour().slots.get(slot_index as usize).cloned();

            let Some(slot) = slot else {
                return stack_left;
            };
            if !slot.has_stack().await {
                return stack_left;
            }

            let mut item = slot.get_cloned_stack().await;
            stack_left = item.clone();

            if slot_index == 3 {
                if !self.insert_item(&mut item, 4, 40, true).await {
                    return ItemStack::EMPTY.clone();
                }
                slot.on_quick_move_crafted(item.clone(), stack_left.clone())
                    .await;
            } else if slot_index != 0 && slot_index != 1 && slot_index != 2 {
                if is_banner(&item) {
                    if !self.insert_item(&mut item, 0, 1, false).await {
                        return ItemStack::EMPTY.clone();
                    }
                } else if is_dye_item(&item) {
                    if !self.insert_item(&mut item, 1, 2, false).await {
                        return ItemStack::EMPTY.clone();
                    }
                } else if is_pattern_item(&item) {
                    if !self.insert_item(&mut item, 2, 3, false).await {
                        return ItemStack::EMPTY.clone();
                    }
                } else if (4..31).contains(&slot_index) {
                    if !self.insert_item(&mut item, 31, 40, false).await {
                        return ItemStack::EMPTY.clone();
                    }
                } else if (31..40).contains(&slot_index)
                    && !self.insert_item(&mut item, 4, 31, false).await
                {
                    return ItemStack::EMPTY.clone();
                }
            } else if !self.insert_item(&mut item, 4, 40, false).await {
                return ItemStack::EMPTY.clone();
            }

            if item.is_empty() {
                slot.set_stack(ItemStack::EMPTY.clone()).await;
            } else {
                slot.mark_dirty().await;
            }

            if item.item_count == stack_left.item_count {
                return ItemStack::EMPTY.clone();
            }

            slot.on_take_item(player, &item).await;
            stack_left
        })
    }
}

/// `mayPlace` for the banner/dye/pattern input slots (LoomMenu.java:64-81).
struct LoomInputSlot {
    inventory: Arc<dyn Inventory>,
    index: usize,
    id: AtomicU8,
    predicate: fn(&ItemStack) -> bool,
}

impl LoomInputSlot {
    fn new(inventory: Arc<dyn Inventory>, index: usize, predicate: fn(&ItemStack) -> bool) -> Self {
        Self {
            inventory,
            index,
            id: AtomicU8::new(0),
            predicate,
        }
    }
}

impl Slot for LoomInputSlot {
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

    fn can_insert<'a>(&'a self, stack: &'a ItemStack) -> BoxFuture<'a, bool> {
        Box::pin(async move { (self.predicate)(stack) })
    }

    fn mark_dirty(&self) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.inventory.mark_dirty();
        })
    }
}

/// Result slot (LoomMenu.java:82-105). Never accepts items; always empty, since result
/// computation is unimplemented (see module docs).
struct LoomResultSlot {
    inventory: Arc<dyn Inventory>,
    index: usize,
    id: AtomicU8,
}

impl LoomResultSlot {
    fn new(inventory: Arc<dyn Inventory>, index: usize) -> Self {
        Self {
            inventory,
            index,
            id: AtomicU8::new(0),
        }
    }
}

impl Slot for LoomResultSlot {
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

    fn can_insert(&self, _stack: &ItemStack) -> BoxFuture<'_, bool> {
        Box::pin(async move { false })
    }

    fn mark_dirty(&self) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.inventory.mark_dirty();
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{build_equipment_slots, entity_equipment::EntityEquipment};
    use pumpkin_data::item::Item;
    use tokio::sync::Mutex as TokioMutex;

    fn handler() -> LoomScreenHandler {
        let player_inventory = Arc::new(PlayerInventory::new(
            Arc::new(TokioMutex::new(EntityEquipment::new())),
            Arc::new(build_equipment_slots()),
        ));
        LoomScreenHandler::new(0, &player_inventory)
    }

    #[test]
    fn banner_predicate_matches_only_banners() {
        assert!(is_banner(&ItemStack::new(1, &Item::WHITE_BANNER)));
        assert!(!is_banner(&ItemStack::new(1, &Item::WHITE_DYE)));
    }

    #[test]
    fn dye_predicate_matches_only_dyes() {
        assert!(is_dye_item(&ItemStack::new(1, &Item::WHITE_DYE)));
        assert!(!is_dye_item(&ItemStack::new(1, &Item::WHITE_BANNER)));
    }

    #[test]
    fn pattern_predicate_matches_only_loom_pattern_items() {
        assert!(is_pattern_item(&ItemStack::new(
            1,
            &Item::FLOWER_BANNER_PATTERN
        )));
        assert!(!is_pattern_item(&ItemStack::new(1, &Item::WHITE_DYE)));
    }

    #[tokio::test]
    async fn banner_slot_rejects_non_banner_items() {
        let handler = handler();
        let slot = handler.get_behaviour().slots[0].clone();
        assert!(!slot.can_insert(&ItemStack::new(1, &Item::WHITE_DYE)).await);
        assert!(
            slot.can_insert(&ItemStack::new(1, &Item::WHITE_BANNER))
                .await
        );
    }

    #[tokio::test]
    async fn result_slot_never_accepts_items() {
        let handler = handler();
        let result_slot = handler.get_behaviour().slots[3].clone();
        assert!(
            !result_slot
                .can_insert(&ItemStack::new(1, &Item::WHITE_BANNER))
                .await
        );
    }
}
