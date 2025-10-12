use crate::block::entities::BlockEntity;
use crate::inventory::{Clearable, Inventory, split_stack};
use crate::item::ItemStack;
use async_trait::async_trait;
use futures::executor::block_on;
use pumpkin_data::item::Item;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use std::any::Any;
use std::array::from_fn;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct ShelfBlockEntity {
    pub position: BlockPos,
    pub items: [Arc<Mutex<ItemStack>>; Self::INVENTORY_SIZE],
    pub dirty: AtomicBool,
    pub align_to_bottom: bool,
}

impl ShelfBlockEntity {
    pub const INVENTORY_SIZE: usize = 3;
    pub const ID: &'static str = "minecraft:shelf";
    pub const ALIGN_ITEMS_TO_BOTTOM_KEY: &'static str = "align_items_to_bottom";

    pub fn new(position: BlockPos) -> Self {
        let item_template = ItemStack::new(1, &Item::REDSTONE);
        let items = from_fn(|_| Arc::new(Mutex::new(item_template.clone())));
        Self {
            position,
            items,
            dirty: AtomicBool::new(false),
            align_to_bottom: false,
        }
    }
}

#[async_trait]
impl BlockEntity for ShelfBlockEntity {
    async fn write_nbt(&self, nbt: &mut NbtCompound) {
        self.write_data(nbt, &self.items, true).await;
        nbt.put_bool(Self::ALIGN_ITEMS_TO_BOTTOM_KEY, self.align_to_bottom);
    }

    fn from_nbt(nbt: &NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let wooden_shelf = Self {
            position,
            items: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY.clone()))),
            dirty: AtomicBool::new(false),
            align_to_bottom: nbt.get_bool(Self::ALIGN_ITEMS_TO_BOTTOM_KEY).unwrap(),
        };
        wooden_shelf.read_data(nbt, &wooden_shelf.items);
        wooden_shelf
    }

    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        nbt.put_bool(Self::ALIGN_ITEMS_TO_BOTTOM_KEY, self.align_to_bottom);
        block_on(self.write_data(&mut nbt, &self.items, true));
        Some(nbt)
    }

    fn get_inventory(self: Arc<Self>) -> Option<Arc<dyn Inventory>> {
        Some(self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
#[async_trait]
impl Inventory for ShelfBlockEntity {
    fn size(&self) -> usize {
        self.items.len()
    }

    async fn is_empty(&self) -> bool {
        for slot in self.items.iter() {
            if !slot.lock().await.is_empty() {
                return false;
            }
        }
        true
    }

    async fn get_stack(&self, slot: usize) -> Arc<Mutex<ItemStack>> {
        self.items[slot].clone()
    }

    async fn remove_stack(&self, slot: usize) -> ItemStack {
        let mut removed = ItemStack::EMPTY.clone();
        let mut guard = self.items[slot].lock().await;
        std::mem::swap(&mut removed, &mut *guard);
        removed
    }

    async fn remove_stack_specific(&self, slot: usize, amount: u8) -> ItemStack {
        split_stack(&self.items, slot, amount).await
    }

    async fn set_stack(&self, slot: usize, stack: ItemStack) {
        *self.items[slot].lock().await = stack;
    }

    fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
impl Clearable for ShelfBlockEntity {
    async fn clear(&self) {
        for slot in self.items.iter() {
            *slot.lock().await = ItemStack::EMPTY.clone();
        }
    }
}
