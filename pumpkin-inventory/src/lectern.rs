//! Lectern screen handler.
//!
//! Vanilla slot layout (LecternMenu.java):
//! - Slot 0: Book slot (written book or writable book only)
//! - Slots 1-36: Player inventory (1-27 main, 28-36 hotbar)
//!
//! Window properties:
//! - 0: Page number (0-based index of the currently displayed page)
//!
//! The lectern is unusual among screen handlers:
//! - The book cannot be shift-clicked out by the player
//! - Page navigation is done via button clicks (handled by block entity)
//! - The "Take Book" action removes the book and closes the screen

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

/// The lectern's 1-slot inventory for holding a book.
pub struct LecternInventory {
    slots: Vec<Arc<Mutex<ItemStack>>>,
    dirty: std::sync::atomic::AtomicBool,
}

impl Default for LecternInventory {
    fn default() -> Self {
        Self::new()
    }
}

impl LecternInventory {
    #[must_use]
    pub fn new() -> Self {
        Self {
            slots: vec![Arc::new(Mutex::new(ItemStack::EMPTY.clone()))],
            dirty: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

impl Inventory for LecternInventory {
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

impl Clearable for LecternInventory {
    fn clear(&self) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            *self.slots[0].lock().await = ItemStack::EMPTY.clone();
        })
    }
}

/// Check if an item is a valid book for the lectern.
#[must_use]
pub fn is_lectern_book(item: &'static Item) -> bool {
    item == &Item::WRITTEN_BOOK || item == &Item::WRITABLE_BOOK
}

/// Book slot (slot 0) — only accepts written books and writable books.
///
/// In vanilla, the lectern book slot prevents shift-click removal.
/// The book can only be taken via the "Take Book" button action.
pub struct LecternBookSlot {
    inventory: Arc<LecternInventory>,
    index: usize,
    id: std::sync::atomic::AtomicU8,
}

impl LecternBookSlot {
    #[must_use]
    pub const fn new(inventory: Arc<LecternInventory>, index: usize) -> Self {
        Self {
            inventory,
            index,
            id: std::sync::atomic::AtomicU8::new(0),
        }
    }
}

impl Slot for LecternBookSlot {
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
        let ok = is_lectern_book(stack.item);
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

/// Lectern screen handler.
///
/// Slot layout:
/// - 0: Book slot (written book or writable book)
/// - 1-36: Player inventory
///
/// Window properties:
/// - 0: Page number (0-based, synced via block entity)
///
/// In vanilla, the lectern screen is read-only from the player's perspective.
/// The book cannot be shift-clicked out — it must be taken via the "Take Book"
/// button, which is handled as a container button click by the block entity.
///
/// TODO: Page navigation button clicks and "Take Book" action require
/// block entity integration (`ButtonClickHandler` in the block code).
pub struct LecternScreenHandler {
    behaviour: ScreenHandlerBehaviour,
    /// The lectern's book inventory. Kept for block entity sync;
    /// the book is NOT dropped on close (stays in the lectern).
    pub inventory: Arc<LecternInventory>,
    /// Current page number (0-based).
    pub page_number: i32,
}

impl LecternScreenHandler {
    #[allow(clippy::unused_async)]
    pub async fn new(sync_id: u8, player_inventory: &Arc<PlayerInventory>) -> Self {
        let inventory = Arc::new(LecternInventory::new());

        let mut handler = Self {
            behaviour: ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Lectern)),
            inventory: inventory.clone(),
            page_number: 0,
        };

        // Slot 0: Book
        handler.add_slot(Arc::new(LecternBookSlot::new(inventory, 0)));

        // Slots 1-36: Player inventory
        let player_inv: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&player_inv);

        handler
    }
}

impl ScreenHandler for LecternScreenHandler {
    fn on_closed<'a>(
        &'a mut self,
        player: &'a dyn InventoryPlayer,
    ) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            self.default_on_closed(player).await;
            // Note: In vanilla, the book stays in the lectern when the screen is closed.
            // It is NOT dropped back to the player like other screen handlers.
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
        _slot_index: i32,
    ) -> ItemStackFuture<'a> {
        // In vanilla, shift-clicking in the lectern screen does nothing.
        // The book cannot be shift-clicked out, and items from the player
        // inventory cannot be shift-clicked into the lectern.
        Box::pin(async move { ItemStack::EMPTY.clone() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lectern_inventory_size() {
        let inv = LecternInventory::new();
        assert_eq!(inv.size(), 1);
    }

    #[test]
    fn lectern_inventory_starts_empty() {
        let inv = LecternInventory::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        assert!(rt.block_on(inv.is_empty()));
    }

    #[test]
    fn lectern_book_slot_accepts_written_book() {
        let inv = Arc::new(LecternInventory::new());
        let slot = LecternBookSlot::new(inv, 0);
        let book = ItemStack::new(1, &Item::WRITTEN_BOOK);
        let rt = tokio::runtime::Runtime::new().unwrap();
        assert!(rt.block_on(slot.can_insert(&book)));
    }

    #[test]
    fn lectern_book_slot_accepts_writable_book() {
        let inv = Arc::new(LecternInventory::new());
        let slot = LecternBookSlot::new(inv, 0);
        let book = ItemStack::new(1, &Item::WRITABLE_BOOK);
        let rt = tokio::runtime::Runtime::new().unwrap();
        assert!(rt.block_on(slot.can_insert(&book)));
    }

    #[test]
    fn lectern_book_slot_rejects_non_book() {
        let inv = Arc::new(LecternInventory::new());
        let slot = LecternBookSlot::new(inv, 0);
        let diamond = ItemStack::new(1, &Item::DIAMOND);
        let rt = tokio::runtime::Runtime::new().unwrap();
        assert!(!rt.block_on(slot.can_insert(&diamond)));
    }

    #[test]
    fn lectern_book_slot_rejects_enchanted_book() {
        // Enchanted books are NOT valid lectern books
        let inv = Arc::new(LecternInventory::new());
        let slot = LecternBookSlot::new(inv, 0);
        let enchanted = ItemStack::new(1, &Item::ENCHANTED_BOOK);
        let rt = tokio::runtime::Runtime::new().unwrap();
        assert!(!rt.block_on(slot.can_insert(&enchanted)));
    }

    #[test]
    fn lectern_book_slot_max_count_is_one() {
        let inv = Arc::new(LecternInventory::new());
        let slot = LecternBookSlot::new(inv, 0);
        let rt = tokio::runtime::Runtime::new().unwrap();
        assert_eq!(rt.block_on(slot.get_max_item_count()), 1);
    }

    #[test]
    fn lectern_inventory_set_and_get() {
        let inv = LecternInventory::new();
        let book = ItemStack::new(1, &Item::WRITTEN_BOOK);
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(inv.set_stack(0, book));
        let stack = rt.block_on(inv.get_stack(0));
        let stack = rt.block_on(stack.lock());
        assert!(stack.item == &Item::WRITTEN_BOOK);
        assert_eq!(stack.item_count, 1);
    }

    #[test]
    fn lectern_is_valid_book_check() {
        assert!(is_lectern_book(&Item::WRITTEN_BOOK));
        assert!(is_lectern_book(&Item::WRITABLE_BOOK));
        assert!(!is_lectern_book(&Item::BOOK));
        assert!(!is_lectern_book(&Item::ENCHANTED_BOOK));
        assert!(!is_lectern_book(&Item::KNOWLEDGE_BOOK));
    }
}
