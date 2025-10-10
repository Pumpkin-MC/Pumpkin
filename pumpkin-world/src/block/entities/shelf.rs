use crate::BlockStateId;
use crate::block::entities::BlockEntity;
use crate::inventory::{Clearable, Inventory, split_stack};
use crate::item::ItemStack;
use async_trait::async_trait;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use std::any::Any;
use std::array::from_fn;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI8};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct ShelfBlockEntity {
    pub position: BlockPos,
    pub items: [Arc<Mutex<ItemStack>>; 3],
    pub dirty: AtomicBool,
}

impl ShelfBlockEntity {
    pub fn new(position: BlockPos, items: [Arc<Mutex<ItemStack>>; 3]) -> Self {
        Self {
            position,
            items: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY.clone()))),
            dirty: AtomicBool::new(false),
        }
    }
}

#[async_trait]
impl BlockEntity for ShelfBlockEntity {
    async fn write_nbt(&self, nbt: &mut NbtCompound) {
        todo!()
    }

    fn from_nbt(nbt: &NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        todo!()
    }

    fn resource_location(&self) -> &'static str {
        todo!()
    }

    fn get_position(&self) -> BlockPos {
        self.position
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
