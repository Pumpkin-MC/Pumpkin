use crate::block::entities::BlockEntity;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::inventory::{Clearable, Inventory, InventoryFuture, sync_write_items_to_nbt};
use rand::{RngExt, rng};
use std::any::Any;
use std::array::from_fn;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct DropperBlockEntity {
    pub position: BlockPos,
    pub items: tokio::sync::RwLock<[ItemStack; Self::INVENTORY_SIZE]>,
    pub dirty: AtomicBool,
}

impl BlockEntity for DropperBlockEntity {
    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        self.write_inventory_nbt(nbt, true)
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let mut dropper = Self {
            position,
            items: tokio::sync::RwLock::new(from_fn(|_| ItemStack::EMPTY.clone())),
            dirty: AtomicBool::new(false),
        };

        pumpkin_world::inventory::sync_read_items_from_nbt(nbt, dropper.items.get_mut());

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
        self.dirty.load(Ordering::Relaxed)
    }

    fn clear_dirty(&self) {
        self.dirty.store(false, Ordering::Relaxed);
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        let items = futures::executor::block_on(self.items.read());
        sync_write_items_to_nbt(items.as_slice(), &mut nbt);
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl DropperBlockEntity {
    pub const INVENTORY_SIZE: usize = 9;
    pub const ID: &'static str = "minecraft:dropper";

    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            items: tokio::sync::RwLock::new(from_fn(|_| ItemStack::EMPTY.clone())),
            dirty: AtomicBool::new(false),
        }
    }

    pub async fn get_random_slot(&self) -> Option<(usize, ItemStack)> {
        let items = self.items.read().await;
        let mut non_empty = Vec::new();
        for (i, stack) in items.iter().enumerate() {
            if !stack.is_empty() {
                non_empty.push((i, stack.clone()));
            }
        }
        if non_empty.is_empty() {
            None
        } else {
            let selected = rng().random_range(0..non_empty.len());
            Some(non_empty[selected].clone())
        }
    }
}

impl Inventory for DropperBlockEntity {
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

impl Clearable for DropperBlockEntity {
    fn clear(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let mut items = self.items.write().await;
            items.fill_with(|| ItemStack::EMPTY.clone());
            self.mark_dirty();
        })
    }
}
