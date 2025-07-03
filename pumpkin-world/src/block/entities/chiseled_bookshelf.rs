use std::{
    array::from_fn,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use async_trait::async_trait;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use tokio::sync::Mutex;

use crate::{
    block::entities::BlockEntity,
    inventory::{Clearable, Inventory, split_stack},
    item::ItemStack,
};

#[derive(Debug)]
pub struct ChiseledBookshelfBlockEntity {
    pub position: BlockPos,
    pub items: [Arc<Mutex<ItemStack>>; 6],
    pub dirty: AtomicBool,
}

#[async_trait]
impl BlockEntity for ChiseledBookshelfBlockEntity {
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
        let chiseled_bookshelf = Self {
            position,
            items: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY))),
            dirty: AtomicBool::new(false),
        };

        chiseled_bookshelf.read_data(nbt, &chiseled_bookshelf.items);

        chiseled_bookshelf
    }

    async fn write_nbt(&self, nbt: &mut NbtCompound) {
        self.write_data(nbt, &self.items, true).await;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ChiseledBookshelfBlockEntity {
    pub const ID: &'static str = "minecraft:chiseled_bookshelf";

    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            items: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY))),
            dirty: AtomicBool::new(false),
        }
    }
}

#[async_trait]
impl Inventory for ChiseledBookshelfBlockEntity {
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
        let mut removed = ItemStack::EMPTY;
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
}

#[async_trait]
impl Clearable for ChiseledBookshelfBlockEntity {
    async fn clear(&self) {
        for slot in self.items.iter() {
            *slot.lock().await = ItemStack::EMPTY;
        }
    }
}
