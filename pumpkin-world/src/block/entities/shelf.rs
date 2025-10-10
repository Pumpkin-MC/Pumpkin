use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI8};
use async_trait::async_trait;
use tokio::sync::Mutex;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use crate::block::entities::BlockEntity;
use crate::BlockStateId;
use crate::inventory::{Clearable, Inventory};
use crate::item::ItemStack;

#[derive(Debug)]
pub struct ShelfBlockEntity {
    pub position: BlockPos,
    pub items: [Arc<Mutex<ItemStack>>; 6],
    pub last_interacted_slot: AtomicI8,
    pub dirty: AtomicBool,
}

#[async_trait]
impl BlockEntity for ShelfBlockEntity {
    async fn write_nbt(&self, nbt: &mut NbtCompound) {
        todo!()
    }

    fn from_nbt(nbt: &NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized
    {
        todo!()
    }

    fn resource_location(&self) -> &'static str {
        todo!()
    }

    fn get_position(&self) -> BlockPos {
        todo!()
    }

    async fn write_internal(&self, nbt: &mut NbtCompound) {
        todo!()
    }

    fn get_inventory(self: Arc<Self>) -> Option<Arc<dyn Inventory>> {
        todo!()
    }

    fn set_block_state(&mut self, _block_state: BlockStateId) {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        todo!()
    }
}
#[async_trait]
impl Inventory for ShelfBlockEntity {
    fn size(&self) -> usize {
        todo!()
    }

    async fn is_empty(&self) -> bool {
        todo!()
    }

    async fn get_stack(&self, slot: usize) -> Arc<Mutex<ItemStack>> {
        todo!()
    }

    async fn remove_stack(&self, slot: usize) -> ItemStack {
        todo!()
    }

    async fn remove_stack_specific(&self, slot: usize, amount: u8) -> ItemStack {
        todo!()
    }

    async fn set_stack(&self, slot: usize, stack: ItemStack) {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        todo!()
    }
}

# [async_trait]
impl Clearable for ShelfBlockEntity {
    async fn clear(&self) {
        todo!()
    }
}