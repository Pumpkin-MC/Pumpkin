use async_trait::async_trait;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::random::xoroshiro128::Xoroshiro;
use pumpkin_util::random::{RandomImpl, get_seed};
use std::any::Any;
use std::sync::Arc;

use crate::block::viewer::{ViewerCountListener, ViewerCountTracker};
use crate::world::SimpleWorld;

use super::BlockEntity;

#[derive(Debug)]
pub struct EnderChestBlockEntity {
    pub position: BlockPos,

    // Viewer
    viewers: Arc<ViewerCountTracker>,
}

#[async_trait]
impl BlockEntity for EnderChestBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(_nbt: &NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        Self {
            position,
            viewers: Arc::new(ViewerCountTracker::new()),
        }
    }

    async fn write_nbt(&self, _nbt: &mut NbtCompound) {}

    async fn tick(&self, world: Arc<dyn SimpleWorld>) {
        self.viewers
            .update_viewer_count::<EnderChestBlockEntity>(self, world, &self.position)
            .await;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
impl ViewerCountListener for EnderChestBlockEntity {
    async fn on_container_open(&self, world: &Arc<dyn SimpleWorld>, _position: &BlockPos) {
        self.play_sound(world, Sound::BlockEnderChestOpen).await;
    }

    async fn on_container_close(&self, world: &Arc<dyn SimpleWorld>, _position: &BlockPos) {
        self.play_sound(world, Sound::BlockEnderChestClose).await;
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

impl EnderChestBlockEntity {
    pub const LID_ANIMATION_EVENT_TYPE: u8 = 1;
    pub const ID: &'static str = "minecraft:ender_chest";

    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            viewers: Arc::new(ViewerCountTracker::new()),
        }
    }

    pub fn get_tracker(&self) -> Arc<ViewerCountTracker> {
        self.viewers.clone()
    }

    async fn play_sound(&self, world: &Arc<dyn SimpleWorld>, sound: Sound) {
        let mut rng = Xoroshiro::from_seed(get_seed());

        world
            .play_sound_fine(
                sound,
                SoundCategory::Blocks,
                &self.position.to_centered_f64(),
                0.5,
                rng.next_f32() * 0.1 + 0.9,
            )
            .await;
    }
}
