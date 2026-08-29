use pumpkin_data::data_component_impl::{WritableBookContentImpl, WrittenBookContentImpl};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::position::BlockPos;
use std::{
    any::Any,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

use crate::block::entities::BlockEntity;
use pumpkin_world::inventory::{Clearable, Inventory};

pub struct LecternBlockEntity {
    pub position: BlockPos,
    pub book: Arc<Mutex<ItemStack>>,
    pub page: AtomicUsize,
    pub dirty: AtomicBool,
}

impl BlockEntity for LecternBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let book_stack = nbt
            .get_compound("Book")
            .and_then(ItemStack::read_item_stack)
            .unwrap_or_else(|| ItemStack::EMPTY.clone());

        let page_count = Self::page_count_of(&book_stack);
        let page = nbt
            .get_int("Page")
            .unwrap_or(0)
            .clamp(0, page_count.saturating_sub(1).max(0)) as usize;
        let book = Arc::new(Mutex::new(book_stack));

        Self {
            position,
            book,
            page: AtomicUsize::new(page),
            dirty: AtomicBool::new(false),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        let book = self
            .book
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !book.is_empty() {
            let mut book_nbt = NbtCompound::default();
            book.write_item_stack(&mut book_nbt);
            nbt.put_compound("Book", book_nbt);
        }
        nbt.put_int("Page", self.page.load(Ordering::Relaxed) as i32);
    }

    fn get_inventory(self: Arc<Self>) -> Option<Arc<dyn Inventory>> {
        Some(self)
    }

    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    fn clear_dirty(&self) {
        self.dirty.store(false, Ordering::Relaxed);
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        if let Ok(book) = self.book.try_lock()
            && !book.is_empty()
        {
            let mut book_nbt = NbtCompound::new();
            book.write_item_stack(&mut book_nbt);
            nbt.put("Book", NbtTag::Compound(book_nbt));
        }
        nbt.put_int("Page", self.page.load(Ordering::Relaxed) as i32);
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl LecternBlockEntity {
    pub const ID: &'static str = "minecraft:lectern";

    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            book: Arc::new(Mutex::new(ItemStack::EMPTY.clone())),
            page: AtomicUsize::new(0),
            dirty: AtomicBool::new(false),
        }
    }

    /// Number of pages in a writable or written book, `0` for anything else.
    #[must_use]
    pub fn page_count_of(stack: &ItemStack) -> i32 {
        stack
            .get_data_component::<WrittenBookContentImpl>()
            .map(|content| content.pages.len())
            .or_else(|| {
                stack
                    .get_data_component::<WritableBookContentImpl>()
                    .map(|content| content.pages.len())
            })
            .map_or(0, |pages| pages as i32)
    }

    pub fn page_count(&self) -> i32 {
        Self::page_count_of(
            &self
                .book
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        )
    }

    /// Vanilla comparator output: `floor(page / (page_count - 1) * 14) + 1`,
    /// or `0` without a book. Single-page books emit `1` (`0 / 0` is `NaN`,
    /// which vanilla's `MathHelper.floor` turns into `0`).
    pub fn comparator_output(&self) -> u8 {
        let book = self
            .book
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if book.is_empty() {
            return 0;
        }

        let page = self.page.load(Ordering::Relaxed) as f32;
        let page_count = Self::page_count_of(&book) as f32;
        let fraction = page / (page_count - 1.0) * 14.0;
        // `NaN as u8` is 0, matching vanilla's cast of NaN to int.
        fraction.floor() as u8 + 1
    }
}

impl Inventory for LecternBlockEntity {
    fn size(&self) -> usize {
        1
    }

    fn is_empty(&self) -> bool {
        self.book
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_empty()
    }

    fn get_stack(&self, _slot: usize) -> ItemStack {
        self.book
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    fn remove_stack(&self, _slot: usize) -> ItemStack {
        let mut removed = ItemStack::EMPTY.clone();
        let mut guard = self
            .book
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        std::mem::swap(&mut removed, &mut *guard);
        self.mark_dirty();
        removed
    }

    fn remove_stack_specific(&self, _slot: usize, amount: u8) -> ItemStack {
        let mut stack = self
            .book
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if stack.is_empty() {
            return ItemStack::EMPTY.clone();
        }
        let res = stack.split(amount);
        self.mark_dirty();
        res
    }

    fn set_stack(&self, _slot: usize, stack: ItemStack) {
        *self
            .book
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = stack;
        // A freshly placed book always opens on its first page.
        self.page.store(0, Ordering::Relaxed);
        self.mark_dirty();
    }

    fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Clearable for LecternBlockEntity {
    fn clear(&self) {
        *self
            .book
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = ItemStack::EMPTY.clone();
        self.mark_dirty();
    }
}
