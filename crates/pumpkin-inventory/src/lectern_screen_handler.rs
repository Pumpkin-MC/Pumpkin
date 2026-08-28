use std::any::Any;
use std::sync::Arc;

use crate::screen_handler::{
    InventoryPlayer, ScreenHandler, ScreenHandlerBehaviour, ScreenProperty, offer_or_drop_stack,
};
use crate::slot::NormalSlot;

use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::screen::WindowType;
use pumpkin_world::block::entities::PropertyDelegate;
use pumpkin_world::inventory::Inventory;

/// Callbacks into the lectern block so page turns and book removal can drive
/// block-state changes (redstone pulse, `has_book`) that live outside this crate.
pub trait LecternController: Send + Sync {
    /// The page currently displayed.
    fn current_page(&self) -> i32;

    /// Clamps and persists `page`, emitting a redstone pulse when it changes.
    fn set_page(&self, page: i32);

    /// Restores the bookless block state after the book was taken.
    fn on_book_taken(&self);
}

/// Exposes the current page as container property 0 (see `window_property::Lectern`).
struct PageDelegate(Arc<dyn LecternController>);

impl PropertyDelegate for PageDelegate {
    fn get_property(&self, index: i32) -> i32 {
        if index == 0 { self.0.current_page() } else { 0 }
    }

    fn set_property(&self, _index: i32, _value: i32) {}

    fn get_properties_size(&self) -> i32 {
        1
    }
}

/// Vanilla `LecternScreenHandler`: a single book slot, no player slots and the
/// current page synced as property 0. Page navigation and taking the book are
/// plain button clicks sent by the client.
pub struct LecternScreenHandler {
    behaviour: ScreenHandlerBehaviour,
    inventory: Arc<dyn Inventory>,
    controller: Arc<dyn LecternController>,
}

impl LecternScreenHandler {
    const PREVIOUS_PAGE_BUTTON_ID: i32 = 1;
    const NEXT_PAGE_BUTTON_ID: i32 = 2;
    const TAKE_BOOK_BUTTON_ID: i32 = 3;
    /// Button ids at or above this jump directly to `id - JUMP_TO_PAGE_OFFSET`.
    const JUMP_TO_PAGE_OFFSET: i32 = 100;

    pub fn new(
        sync_id: u8,
        inventory: Arc<dyn Inventory>,
        controller: Arc<dyn LecternController>,
    ) -> Self {
        let mut handler = Self {
            behaviour: ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Lectern)),
            inventory: inventory.clone(),
            controller: controller.clone(),
        };

        handler.add_slot(Arc::new(NormalSlot::new(inventory, 0)));
        handler.add_property(ScreenProperty::new(Arc::new(PageDelegate(controller)), 0));

        handler
    }
}

impl ScreenHandler for LecternScreenHandler {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_behaviour(&self) -> &ScreenHandlerBehaviour {
        &self.behaviour
    }

    fn get_behaviour_mut(&mut self) -> &mut ScreenHandlerBehaviour {
        &mut self.behaviour
    }

    fn on_button_click(&mut self, player: &dyn InventoryPlayer, id: i32) -> bool {
        match id {
            Self::PREVIOUS_PAGE_BUTTON_ID => {
                self.controller.set_page(self.controller.current_page() - 1);
                true
            }
            Self::NEXT_PAGE_BUTTON_ID => {
                self.controller.set_page(self.controller.current_page() + 1);
                true
            }
            Self::TAKE_BOOK_BUTTON_ID => {
                let stack = self.inventory.remove_stack(0);
                if stack.is_empty() {
                    return false;
                }
                self.inventory.mark_dirty();
                self.controller.on_book_taken();
                offer_or_drop_stack(player, stack);
                self.send_content_updates();
                true
            }
            _ if id >= Self::JUMP_TO_PAGE_OFFSET => {
                self.controller.set_page(id - Self::JUMP_TO_PAGE_OFFSET);
                true
            }
            _ => false,
        }
    }

    fn quick_move(&mut self, _player: &dyn InventoryPlayer, _slot_index: i32) -> ItemStack {
        // The lectern screen has no player slots, so nothing can be shift-clicked.
        ItemStack::EMPTY.clone()
    }
}
