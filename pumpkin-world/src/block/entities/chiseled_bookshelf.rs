use log::warn;
use pumpkin_data::Block;
use pumpkin_data::block_properties::{BlockProperties, ChiseledBookshelfLikeProperties};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use std::any::Any;
use std::pin::Pin;
use std::{
    array::from_fn,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicI8, Ordering},
    },
};
use tokio::sync::Mutex;

use crate::inventory::InventoryFuture;
use crate::{
    block::entities::BlockEntity,
    inventory::{Clearable, Inventory, split_stack},
    item::ItemStack,
    world::{BlockFlags, SimpleWorld},
};

pub struct ChiseledBookshelfBlockEntity {
    pub position: BlockPos,
    pub items: [Arc<Mutex<ItemStack>>; Self::INVENTORY_SIZE],
    pub last_interacted_slot: AtomicI8,
    pub dirty: AtomicBool,
}

const LAST_INTERACTED_SLOT: &str = "last_interacted_slot";

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
            items: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY.clone()))),
            last_interacted_slot: AtomicI8::new(
                nbt.get_int(LAST_INTERACTED_SLOT).unwrap_or(-1) as i8
            ),
            dirty: AtomicBool::new(false),
        };

        chiseled_bookshelf.read_data(nbt, &chiseled_bookshelf.items);

        chiseled_bookshelf
    }

    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            self.write_data(nbt, &self.items, true).await;
            nbt.put_int(
                LAST_INTERACTED_SLOT,
                self.last_interacted_slot.load(Ordering::Relaxed).into(),
            );
        })
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

impl ChiseledBookshelfBlockEntity {
    pub const INVENTORY_SIZE: usize = 6;
    pub const ID: &'static str = "minecraft:chiseled_bookshelf";

    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            items: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY.clone()))),
            last_interacted_slot: AtomicI8::new(-1),
            dirty: AtomicBool::new(false),
        }
    }

    pub async fn update_state(
        &self,
        mut properties: ChiseledBookshelfLikeProperties,
        world: Arc<dyn SimpleWorld>,
        slot: i8,
    ) {
        if slot >= 0 && slot < self.items.len() as i8 {
            self.last_interacted_slot.store(slot, Ordering::Relaxed);

            properties.slot_0_occupied = !self.items[0].lock().await.is_empty();
            properties.slot_1_occupied = !self.items[1].lock().await.is_empty();
            properties.slot_2_occupied = !self.items[2].lock().await.is_empty();
            properties.slot_3_occupied = !self.items[3].lock().await.is_empty();
            properties.slot_4_occupied = !self.items[4].lock().await.is_empty();
            properties.slot_5_occupied = !self.items[5].lock().await.is_empty();

            world
                .set_block_state(
                    &self.position,
                    properties.to_state_id(&Block::CHISELED_BOOKSHELF),
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

impl Inventory for ChiseledBookshelfBlockEntity {
    fn size(&self) -> usize {
        self.items.len()
    }

    fn is_empty(&self) -> InventoryFuture<'_, bool> {
        Box::pin(async move {
            for slot in self.items.iter() {
                if !slot.lock().await.is_empty() {
                    return false;
                }
            }

            true
        })
    }

    fn get_stack(&self, slot: usize) -> InventoryFuture<'_, Arc<Mutex<ItemStack>>> {
        Box::pin(async move { self.items[slot].clone() })
    }

    fn remove_stack(&self, slot: usize) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            let mut removed = ItemStack::EMPTY.clone();
            let mut guard = self.items[slot].lock().await;
            std::mem::swap(&mut removed, &mut *guard);
            removed
        })
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move { split_stack(&self.items, slot, amount).await })
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            *self.items[slot].lock().await = stack;
        })
    }

    fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Clearable for ChiseledBookshelfBlockEntity {
    fn clear(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            for slot in self.items.iter() {
                *slot.lock().await = ItemStack::EMPTY.clone();
            }
        })
    }
}
