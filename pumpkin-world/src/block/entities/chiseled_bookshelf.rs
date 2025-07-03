use std::{
    array::from_fn,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicI8, Ordering},
    },
};

use async_trait::async_trait;
use log::warn;
use pumpkin_data::block_properties::{BlockProperties, ChiseledBookshelfLikeProperties};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use tokio::sync::Mutex;

use crate::{
    block::entities::BlockEntity,
    inventory::{Clearable, Inventory, split_stack},
    item::ItemStack,
    world::{BlockFlags, SimpleWorld},
};

#[derive(Debug)]
pub struct ChiseledBookshelfBlockEntity {
    pub position: BlockPos,
    pub items: [Arc<Mutex<ItemStack>>; 6],
    pub last_interacted_slot: AtomicI8,
    pub dirty: AtomicBool,
}

const LAST_INTERACTED_SLOT: &str = "last_interacted_slot";

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
            last_interacted_slot: AtomicI8::new(
                nbt.get_int(LAST_INTERACTED_SLOT).unwrap_or(-1) as i8
            ),
            dirty: AtomicBool::new(false),
        };

        chiseled_bookshelf.read_data(nbt, &chiseled_bookshelf.items);

        chiseled_bookshelf
    }

    async fn write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_int(
            LAST_INTERACTED_SLOT,
            self.last_interacted_slot.load(Ordering::Relaxed).into(),
        );
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
            last_interacted_slot: AtomicI8::new(-1),
            dirty: AtomicBool::new(false),
        }
    }

    pub async fn update_state(&self, world: Arc<dyn SimpleWorld>, slot: i8) {
        if slot >= 0 && slot < self.items.len() as i8 {
            self.last_interacted_slot.store(slot, Ordering::Relaxed);

            let block = world.get_block(&self.position).await;
            let state = world.get_block_state(&self.position).await;
            let mut properties = ChiseledBookshelfLikeProperties::from_state_id(state.id, block);

            properties.slot_0_occupied = !self.items[0].lock().await.is_empty();
            properties.slot_1_occupied = !self.items[1].lock().await.is_empty();
            properties.slot_2_occupied = !self.items[2].lock().await.is_empty();
            properties.slot_3_occupied = !self.items[3].lock().await.is_empty();
            properties.slot_4_occupied = !self.items[4].lock().await.is_empty();
            properties.slot_5_occupied = !self.items[5].lock().await.is_empty();

            world
                .set_block_state(
                    &self.position,
                    properties.to_state_id(block),
                    BlockFlags::NOTIFY_ALL,
                )
                .await;
        } else {
            warn!(
                "Invalid interacted slot: {} for chiseled bookshelf at position {:?}",
                slot, self.position
            );
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
