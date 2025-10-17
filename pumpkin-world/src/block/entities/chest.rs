use std::{
    any::Any,
    array::from_fn,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use async_trait::async_trait;
use pumpkin_data::{
    Block, HorizontalFacingExt,
    block_properties::{BlockProperties, ChestLikeProperties, ChestType},
    sound::{Sound, SoundCategory},
};
use pumpkin_util::{
    math::{position::BlockPos, vector3::Vector3},
    random::{RandomImpl, get_seed, xoroshiro128::Xoroshiro},
};
use tokio::sync::Mutex;

use crate::{
    block::viewer::{ViewerCountListener, ViewerCountTracker},
    inventory::{Clearable, Inventory, split_stack},
    item::ItemStack,
    world::SimpleWorld,
};

use super::BlockEntity;

#[derive(Debug)]
pub struct ChestBlockEntity {
    pub position: BlockPos,
    pub items: [Arc<Mutex<ItemStack>>; Self::INVENTORY_SIZE],
    pub dirty: AtomicBool,
    pub id: String,

    // Viewer
    viewers: ViewerCountTracker,
}

#[async_trait]
impl BlockEntity for ChestBlockEntity {
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
        let chest = Self {
            position,
            items: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY.clone()))),
            dirty: AtomicBool::new(false),
            viewers: ViewerCountTracker::new(),
            id: String::from(nbt.get_string("id").unwrap()),
        };

        chest.read_data(nbt, &chest.items);
        chest
    }

    async fn write_nbt(&self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        self.write_data(nbt, &self.items, true).await;
        // Safety precaution
        //self.clear().await;
    }

    async fn tick(&self, world: Arc<dyn SimpleWorld>) {
        self.viewers
            .update_viewer_count::<ChestBlockEntity>(self, world, &self.position)
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
impl ViewerCountListener for ChestBlockEntity {
    async fn on_container_open(&self, world: &Arc<dyn SimpleWorld>, _position: &BlockPos) {
        self.play_sound(world, Sound::BlockChestOpen).await;
    }

    async fn on_container_close(&self, world: &Arc<dyn SimpleWorld>, _position: &BlockPos) {
        self.play_sound(world, Sound::BlockChestClose).await;
    }

    async fn on_viewer_count_update(
        &self,
        world: &Arc<dyn SimpleWorld>,
        position: &BlockPos,
        _old: u16,
        new: u16,
    ) {
        world
            .add_synced_block_event(*position, Self::LID_ANIMATION_EVENT_TYPE, new as u8)
            .await
    }
}

impl ChestBlockEntity {
    pub const INVENTORY_SIZE: usize = 27;
    pub const LID_ANIMATION_EVENT_TYPE: u8 = 1;
    pub const ID: &'static str = "minecraft:chest";

    pub fn new(position: BlockPos, id: &str) -> Self {
        Self {
            position,
            items: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY.clone()))),
            dirty: AtomicBool::new(false),
            viewers: ViewerCountTracker::new(),
            id: String::from(id),
        }
    }

    async fn play_sound(&self, world: &Arc<dyn SimpleWorld>, sound: Sound) {
        let mut rng = Xoroshiro::from_seed(get_seed());

        let state = world.get_block_state(&self.position).await;
        let properties = ChestLikeProperties::from_state_id(
            state.id,
            &Block::from_name(self.id.as_str()).unwrap(),
        );
        let position = match properties.r#type {
            ChestType::Left => return,
            ChestType::Single => Vector3::new(
                self.position.0.x as f64 + 0.5,
                self.position.0.y as f64 + 0.5,
                self.position.0.z as f64 + 0.5,
            ),
            ChestType::Right => {
                let direction = properties.facing.to_block_direction().to_offset();
                Vector3::new(
                    self.position.0.x as f64 + 0.5 + direction.x as f64 * 0.5,
                    self.position.0.y as f64 + 0.5,
                    self.position.0.z as f64 + 0.5 + direction.z as f64 * 0.5,
                )
            }
        };

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
impl Inventory for ChestBlockEntity {
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
        self.dirty.store(true, Ordering::Relaxed);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
impl Clearable for ChestBlockEntity {
    async fn clear(&self) {
        for slot in self.items.iter() {
            *slot.lock().await = ItemStack::EMPTY.clone();
        }
    }
}
