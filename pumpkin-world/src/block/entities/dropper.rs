use std::array::from_fn;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use async_trait::async_trait;
use tokio::sync::Mutex;
use pumpkin_util::math::position::BlockPos;
use crate::block::entities::BlockEntity;
use crate::inventory::{split_stack, Clearable, Inventory};
use crate::item::ItemStack;

#[derive(Debug)]
pub struct DropperBlockEntity {
    pub position: BlockPos,
    pub items: [Arc<Mutex<ItemStack>>; 9],
    pub dirty: AtomicBool,
}

#[async_trait]
impl BlockEntity for DropperBlockEntity {
    async fn write_nbt(&self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        self.write_data(nbt, &self.items, true).await;
        // Safety precaution
        //self.clear().await;
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let dropper = Self {
            position,
            items: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY))),
            dirty: AtomicBool::new(false),
        };

        dropper.read_data(nbt, &dropper.items);

        dropper
    }

    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn get_inventory(self: Arc<Self>) -> Option<Arc<dyn Inventory>> {
        Some(self)
    }

    fn is_dirty(&self) -> bool {
        self.dirty.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl DropperBlockEntity {
    pub const ID: &'static str = "minecraft:dropper";
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            items: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY))),
            dirty: AtomicBool::new(false),
        }
    }
}

#[async_trait]
impl Inventory for DropperBlockEntity {
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
        self.dirty.store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

#[async_trait]
impl Clearable for DropperBlockEntity {
    async fn clear(&self) {
        for slot in self.items.iter() {
            *slot.lock().await = ItemStack::EMPTY;
        }
    }
}
