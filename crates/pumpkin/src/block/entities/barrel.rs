use pumpkin_data::block_properties::{BarrelLikeProperties, BlockProperties};
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{Block, FacingExt, item_stack::ItemStack};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::xoroshiro128::Xoroshiro;
use pumpkin_util::random::{RandomImpl, get_seed};
use std::any::Any;
use std::pin::Pin;
use std::{
    array::from_fn,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::block::viewer::{
    ViewerCountListener, ViewerCountTracker, ViewerCountTrackerExt, ViewerFuture,
};
use crate::world::{BlockFlags, World};
use pumpkin_world::inventory::InventoryFuture;
use pumpkin_world::inventory::{Clearable, Inventory, sync_write_items_to_nbt};

use super::BlockEntity;

pub struct BarrelBlockEntity {
    pub position: BlockPos,
    pub items: tokio::sync::RwLock<[ItemStack; Self::INVENTORY_SIZE]>,
    pub dirty: AtomicBool,

    // Viewer
    viewers: ViewerCountTracker,
}

impl BlockEntity for BarrelBlockEntity {
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
        let mut barrel = Self {
            position,
            items: tokio::sync::RwLock::new(from_fn(|_| ItemStack::EMPTY.clone())),
            dirty: AtomicBool::new(false),
            viewers: ViewerCountTracker::new(),
        };

        pumpkin_world::inventory::sync_read_items_from_nbt(nbt, barrel.items.get_mut());

        barrel
    }

    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        self.write_inventory_nbt(nbt, true)
    }

    fn tick<'a>(&'a self, world: &'a Arc<World>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            self.viewers
                .update_viewer_count::<Self>(self, world, &self.position)
                .await;
        })
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
        if let Ok(guard) = self.items.try_read() {
            sync_write_items_to_nbt(&*guard, &mut nbt);
        }
        Some(nbt)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl ViewerCountListener for BarrelBlockEntity {
    fn on_container_open<'a>(
        &'a self,
        world: &'a Arc<World>,
        _position: &'a BlockPos,
    ) -> ViewerFuture<'a, ()> {
        Box::pin(async move {
            self.play_sound(world, Sound::BlockBarrelOpen);
            self.set_open(world, true).await;
        })
    }

    fn on_container_close<'a>(
        &'a self,
        world: &'a Arc<World>,
        _position: &'a BlockPos,
    ) -> ViewerFuture<'a, ()> {
        Box::pin(async move {
            self.play_sound(world, Sound::BlockBarrelClose);
            self.set_open(world, false).await;
        })
    }
}

impl BarrelBlockEntity {
    pub const INVENTORY_SIZE: usize = 27;
    pub const ID: &'static str = "minecraft:barrel";

    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            items: tokio::sync::RwLock::new(from_fn(|_| ItemStack::EMPTY.clone())),
            dirty: AtomicBool::new(false),
            viewers: ViewerCountTracker::new(),
        }
    }

    async fn set_open(&self, world: &Arc<World>, open: bool) {
        let state = world.get_block_state(&self.position);
        let mut properties = BarrelLikeProperties::from_state_id(state.id, &Block::BARREL);

        properties.open = open;

        world
            .clone()
            .set_block_state(
                &self.position,
                properties.to_state_id(&Block::BARREL),
                BlockFlags::NOTIFY_ALL,
            )
            .await;
    }

    fn play_sound(&self, world: &Arc<World>, sound: Sound) {
        let mut rng = Xoroshiro::from_seed(get_seed());

        let state = world.get_block_state(&self.position);
        let properties = BarrelLikeProperties::from_state_id(state.id, &Block::BARREL);
        let direction = properties.facing.to_block_direction().to_offset();
        let position = Vector3::new(
            self.position.0.x as f64 + 0.5 + direction.x as f64 / 2.0,
            self.position.0.y as f64 + 0.5 + direction.y as f64 / 2.0,
            self.position.0.z as f64 + 0.5 + direction.z as f64 / 2.0,
        );
        world.play_sound_fine(
            sound,
            SoundCategory::Blocks,
            &position,
            0.5,
            rng.next_f32() * 0.1 + 0.9,
        );
    }
}

impl Inventory for BarrelBlockEntity {
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

    fn on_open(&self) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            self.viewers.open_container();
        })
    }

    fn on_close(&self) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            self.viewers.close_container();
        })
    }

    fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Clearable for BarrelBlockEntity {
    fn clear(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let mut items = self.items.write().await;
            items.fill_with(|| ItemStack::EMPTY.clone());
            self.mark_dirty();
        })
    }
}
