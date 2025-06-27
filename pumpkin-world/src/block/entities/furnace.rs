use std::{
    array::from_fn,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicI16, Ordering},
    },
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
pub struct FurnaceBlockEntity {
    pub position: BlockPos,
    pub dirty: AtomicBool,

    pub cooking_time_spent: AtomicI16,
    pub cooking_total_time: AtomicI16,
    pub lit_time_remaining: AtomicI16,
    pub lit_total_time: AtomicI16,

    pub items: [Arc<Mutex<ItemStack>>; 3],
}

#[async_trait]
impl BlockEntity for FurnaceBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let cooking_total_time = AtomicI16::new(
            nbt.get_short("cooking_total_time")
                .map_or(0, |cooking_total_time| cooking_total_time),
        );
        let cooking_time_spent = AtomicI16::new(
            nbt.get_short("cooking_time_spent")
                .map_or(0, |cooking_time_spent| cooking_time_spent),
        );
        let lit_total_time = AtomicI16::new(
            nbt.get_short("lit_total_time")
                .map_or(0, |lit_total_time| lit_total_time),
        );
        let lit_time_remaining = AtomicI16::new(
            nbt.get_short("lit_time_remaining")
                .map_or(0, |lit_time_remaining| lit_time_remaining),
        );

        let furnace = Self {
            position,
            dirty: AtomicBool::new(false),
            items: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY))),
            cooking_total_time,
            cooking_time_spent,
            lit_total_time,
            lit_time_remaining,
        };
        furnace.read_data(nbt, &furnace.items);

        furnace
    }

    async fn write_nbt(&self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        nbt.put_short(
            "cooking_total_time",
            self.cooking_total_time.load(Ordering::Relaxed),
        );
        nbt.put_short(
            "cooking_time_spent",
            self.cooking_time_spent.load(Ordering::Relaxed),
        );
        nbt.put_short(
            "lit_total_time",
            self.lit_total_time.load(Ordering::Relaxed),
        );
        nbt.put_short(
            "lit_time_remaining",
            self.lit_time_remaining.load(Ordering::Relaxed),
        );
        self.write_data(nbt, &self.items, true).await;
        // Safety precaution
        // self.clear().await;
    }

    fn get_inventory(self: Arc<Self>) -> Option<Arc<dyn Inventory>> {
        Some(self)
    }

    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl FurnaceBlockEntity {
    pub const ID: &'static str = "minecraft:furnace";
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            dirty: AtomicBool::new(false),
            items: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY))),
            cooking_total_time: AtomicI16::new(0),
            cooking_time_spent: AtomicI16::new(0),
            lit_total_time: AtomicI16::new(0),
            lit_time_remaining: AtomicI16::new(0),
        }
    }
}

#[async_trait]
impl Inventory for FurnaceBlockEntity {
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
impl Clearable for FurnaceBlockEntity {
    async fn clear(&self) {
        for slot in self.items.iter() {
            *slot.lock().await = ItemStack::EMPTY;
        }
    }
}
