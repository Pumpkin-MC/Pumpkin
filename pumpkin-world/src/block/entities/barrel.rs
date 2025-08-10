use async_trait::async_trait;
use pumpkin_data::block_properties::{BarrelLikeProperties, BlockProperties};
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{Block, FacingExt};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::xoroshiro128::Xoroshiro;
use pumpkin_util::random::{RandomImpl, get_seed};
use std::any::Any;
use std::{
    array::from_fn,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::sync::Mutex;

use crate::block::viewer::{ViewerCountListener, ViewerCountTracker};
use crate::world::{BlockFlags, SimpleWorld};
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

    // Viewer
    pub viewers: ViewerCountTracker,
}

#[async_trait]
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
        let barrel = Self {
            position,
            items: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY.clone()))),
            dirty: AtomicBool::new(false),
            viewers: ViewerCountTracker::new(),
        };

        barrel.read_data(nbt, &barrel.items);

        barrel
    }

    async fn write_nbt(&self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        self.write_data(nbt, &self.items, true).await;
        // Safety precaution
        //self.clear().await;
    }

    async fn tick(&self, world: Arc<dyn SimpleWorld>) {
        self.viewers
            .update_viewer_count::<BarrelBlockEntity>(self, world, &self.position)
            .await;
    }

    fn get_inventory(self: Arc<Self>) -> Option<Arc<dyn Inventory>> {
        Some(self)
    }

    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
impl ViewerCountListener for BarrelBlockEntity {
    async fn on_container_open(&self, world: &Arc<dyn SimpleWorld>, _position: &BlockPos) {
        self.play_sound(world, Sound::BlockBarrelOpen).await;
        self.set_open(world, true).await;
    }

    async fn on_container_close(&self, world: &Arc<dyn SimpleWorld>, _position: &BlockPos) {
        self.play_sound(world, Sound::BlockBarrelClose).await;
        self.set_open(world, false).await;
    }
}

impl BarrelBlockEntity {
    pub const ID: &'static str = "minecraft:barrel";
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            items: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY.clone()))),
            dirty: AtomicBool::new(false),
            viewers: ViewerCountTracker::new(),
        }
    }

    async fn set_open(&self, world: &Arc<dyn SimpleWorld>, open: bool) {
        let state = world.get_block_state(&self.position).await;
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

    async fn play_sound(&self, world: &Arc<dyn SimpleWorld>, sound: Sound) {
        let mut rng = Xoroshiro::from_seed(get_seed());

        let state = world.get_block_state(&self.position).await;
        let properties = BarrelLikeProperties::from_state_id(state.id, &Block::BARREL);
        let direction = properties.facing.to_block_direction().to_offset();
        let position = Vector3::new(
            self.position.0.x as f64 + 0.5 + direction.x as f64 / 2.0,
            self.position.0.y as f64 + 0.5 + direction.y as f64 / 2.0,
            self.position.0.z as f64 + 0.5 + direction.z as f64 / 2.0,
        );
        world
            .play_sound_fine(
                sound,
                SoundCategory::Blocks,
                &position,
                0.5,
                rng.next_f32() * 0.1 + 0.9,
            )
            .await;
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

    fn on_open(&self) {
        self.viewers.open_container();
    }

    fn on_close(&self) {
        self.viewers.close_container();
    }

    fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
impl Clearable for BarrelBlockEntity {
    async fn clear(&self) {
        for slot in self.items.iter() {
            *slot.lock().await = ItemStack::EMPTY.clone();
        }
    }
}
