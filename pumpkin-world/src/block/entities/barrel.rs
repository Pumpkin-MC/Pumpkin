use std::{
    array::from_fn,
    sync::{Arc, atomic::AtomicBool},
};

use async_trait::async_trait;
use pumpkin_util::math::position::BlockPos;
use tokio::sync::Mutex;

use crate::{
    inventory::{
        split_stack, {Clearable, Inventory},
    },
    item::ItemStack,
};

use super::BlockEntity;

#[derive(Debug)]
pub struct BarrelBlockEntity {
    pub position: BlockPos,
    pub items: [Arc<Mutex<ItemStack>>; 27],
    pub dirty: AtomicBool,
}

#[async_trait]
impl BlockEntity for BarrelBlockEntity {
    fn identifier(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let barrel = Self {
            position,
            items: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY))),
            dirty: AtomicBool::new(false),
        };

        barrel.read_data(nbt, &barrel.items);

        barrel
    }

    async fn write_nbt(&self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        self.write_data(nbt, &self.items, true).await;
        // Safety precaution
        //self.clear().await;
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

impl BarrelBlockEntity {
    pub const ID: &'static str = "minecraft:barrel";
    pub fn new(position: BlockPos) -> Self {
        println!("Creating barrel");
        Self {
            position,
            items: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY))),
            dirty: AtomicBool::new(false),
        }
    }
}

#[async_trait]
impl Inventory for BarrelBlockEntity {
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
impl Clearable for BarrelBlockEntity {
    async fn clear(&self) {
        for slot in self.items.iter() {
            *slot.lock().await = ItemStack::EMPTY;
        }
    }
}
