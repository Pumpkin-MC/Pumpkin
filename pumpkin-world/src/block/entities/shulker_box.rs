use async_trait::async_trait;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::random::xoroshiro128::Xoroshiro;
use pumpkin_util::random::{RandomImpl, get_seed};
use std::any::Any;
use std::{
    array::from_fn,
    sync::{Arc, atomic::AtomicBool},
};
use tokio::sync::Mutex;

use crate::block::viewer::{ViewerCountListener, ViewerCountTracker};
use crate::world::SimpleWorld;
use crate::{
    inventory::{
        split_stack, {Clearable, Inventory},
    },
    item::ItemStack,
};

use super::BlockEntity;

#[derive(Debug)]
pub struct ShulkerBoxBlockEntity {
    pub position: BlockPos,
    pub items: [Arc<Mutex<ItemStack>>; Self::INVENTORY_SIZE],
    pub dirty: AtomicBool,

    // Viewer
    viewers: ViewerCountTracker,
}

#[async_trait]
impl BlockEntity for ShulkerBoxBlockEntity {
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
        let shulker_box = Self {
            position,
            items: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY.clone()))),
            dirty: AtomicBool::new(false),
            viewers: ViewerCountTracker::new(),
        };

        shulker_box.read_data(nbt, &shulker_box.items);

        shulker_box
    }

    async fn write_nbt(&self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        self.write_data(nbt, &self.items, true).await;
        // Safety precaution
        //self.clear().await;
    }

    async fn tick(&self, world: Arc<dyn SimpleWorld>) {
        self.viewers
            .update_viewer_count::<ShulkerBoxBlockEntity>(self, world, &self.position)
            .await;
    }

    async fn on_block_replaced(self: Arc<Self>, _world: Arc<dyn SimpleWorld>, _position: BlockPos) {
        // Do nothing
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

#[async_trait]
impl ViewerCountListener for ShulkerBoxBlockEntity {
    async fn on_container_open(&self, world: &Arc<dyn SimpleWorld>, position: &BlockPos) {
        self.play_sound(world, position, Sound::BlockShulkerBoxOpen)
            .await;
        // TODO: this.world.emitGameEvent(player, GameEvent.CONTAINER_OPEN, this.pos);
    }

    async fn on_container_close(&self, world: &Arc<dyn SimpleWorld>, position: &BlockPos) {
        self.play_sound(world, position, Sound::BlockShulkerBoxClose)
            .await;
        // TODO: this.world.emitGameEvent(player, GameEvent.CONTAINER_CLOSE, this.pos);
    }

    async fn on_viewer_count_update(
        &self,
        world: &Arc<dyn SimpleWorld>,
        position: &BlockPos,
        _old: u16,
        new: u16,
    ) {
        world
            .add_synced_block_event(*position, Self::OPEN_ANIMATION_EVENT_TYPE, new as u8)
            .await
    }
}

impl ShulkerBoxBlockEntity {
    pub const INVENTORY_SIZE: usize = 27;
    pub const OPEN_ANIMATION_EVENT_TYPE: u8 = 1;
    pub const ID: &'static str = "minecraft:shulker_box"; // TODO support multi IDs

    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            items: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY.clone()))),
            dirty: AtomicBool::new(false),
            viewers: ViewerCountTracker::new(),
        }
    }

    async fn play_sound(&self, world: &Arc<dyn SimpleWorld>, position: &BlockPos, sound: Sound) {
        let mut rng = Xoroshiro::from_seed(get_seed());

        world
            .play_sound_fine(
                sound,
                SoundCategory::Blocks,
                &position.to_centered_f64(),
                0.5,
                rng.next_f32() * 0.1 + 0.9,
            )
            .await;
    }
}

#[async_trait]
impl Inventory for ShulkerBoxBlockEntity {
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

    async fn on_open(&self) {
        self.viewers.open_container();
    }

    async fn on_close(&self) {
        self.viewers.close_container();
    }

    fn mark_dirty(&self) {
        self.dirty.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
impl Clearable for ShulkerBoxBlockEntity {
    async fn clear(&self) {
        for slot in self.items.iter() {
            *slot.lock().await = ItemStack::EMPTY.clone();
        }
    }
}
