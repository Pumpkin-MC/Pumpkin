use pumpkin_data::Block;
use pumpkin_data::block_properties::{BlockProperties, ChiseledBookshelfLikeProperties};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use std::{
    array::from_fn,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicI8, Ordering},
    },
};
use tracing::warn;

use crate::{
    block::entities::BlockEntity,
    world::{BlockFlags, World},
};
use pumpkin_world::inventory::InventoryFuture;
use pumpkin_world::inventory::{Clearable, Inventory, sync_write_items_to_nbt};

pub struct ChiseledBookshelfBlockEntity {
    pub position: BlockPos,
    pub items: tokio::sync::RwLock<[ItemStack; Self::INVENTORY_SIZE]>,
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
        let mut bookshelf = Self {
            position,
            items: tokio::sync::RwLock::new(from_fn(|_| ItemStack::EMPTY.clone())),
            last_interacted_slot: AtomicI8::new(-1),
            dirty: AtomicBool::new(false),
        };
        pumpkin_world::inventory::sync_read_items_from_nbt(nbt, bookshelf.items.get_mut());
        if let Some(slot) = nbt.get_int(LAST_INTERACTED_SLOT) {
            bookshelf
                .last_interacted_slot
                .store(slot as i8, Ordering::Relaxed);
        }

        bookshelf
    }

    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let items = self.items.read().await;
            sync_write_items_to_nbt(items.as_slice(), nbt);
            nbt.put_int(
                LAST_INTERACTED_SLOT,
                i32::from(self.last_interacted_slot.load(Ordering::Relaxed)),
            );
        })
    }

    fn get_inventory(self: Arc<Self>) -> Option<Arc<dyn Inventory>> {
        Some(self as Arc<dyn Inventory>)
    }

    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    fn clear_dirty(&self) {
        self.dirty.store(false, Ordering::Relaxed);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ChiseledBookshelfBlockEntity {
    pub const INVENTORY_SIZE: usize = 6;
    pub const ID: &'static str = "minecraft:chiseled_bookshelf";

    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            items: tokio::sync::RwLock::new(from_fn(|_| ItemStack::EMPTY.clone())),
            last_interacted_slot: AtomicI8::new(-1),
            dirty: AtomicBool::new(false),
        }
    }

    pub async fn update_state(
        &self,
        mut properties: ChiseledBookshelfLikeProperties,
        world: Arc<World>,
        slot: usize,
    ) {
        if (0..Self::INVENTORY_SIZE).contains(&slot) {
            self.last_interacted_slot
                .store(slot as i8, Ordering::Relaxed);
            self.mark_dirty();

            let occupied = !self.get_stack(slot).await.is_empty();
            match slot {
                0 => properties.slot_0_occupied = occupied,
                1 => properties.slot_1_occupied = occupied,
                2 => properties.slot_2_occupied = occupied,
                3 => properties.slot_3_occupied = occupied,
                4 => properties.slot_4_occupied = occupied,
                5 => properties.slot_5_occupied = occupied,
                _ => {}
            }

            world
                .set_block_state(
                    &self.position,
                    properties.to_state_id(&Block::CHISELED_BOOKSHELF),
                    BlockFlags::NOTIFY_LISTENERS,
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
        Self::INVENTORY_SIZE
    }

    fn is_empty(&self) -> InventoryFuture<'_, bool> {
        Box::pin(async move {
            let items = self.items.read().await;
            items.iter().all(ItemStack::is_empty)
        })
    }

    fn get_stack(&self, slot: usize) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            let items = self.items.read().await;
            items[slot].clone()
        })
    }

    fn remove_stack(&self, slot: usize) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            let mut items = self.items.write().await;
            let removed = std::mem::replace(&mut items[slot], ItemStack::EMPTY.clone());
            self.mark_dirty();
            removed
        })
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            let mut items = self.items.write().await;
            let res = if !items[slot].is_empty() && amount > 0 {
                items[slot].split(amount)
            } else {
                ItemStack::EMPTY.clone()
            };
            self.mark_dirty();
            res
        })
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            let mut items = self.items.write().await;
            items[slot] = stack;
            self.mark_dirty();
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
            let mut items = self.items.write().await;
            items.fill_with(|| ItemStack::EMPTY.clone());
            self.mark_dirty();
        })
    }
}
