use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_inventory::viewer_count_tracker::{
    ViewerCountListener, ViewerCountTracker, ViewerFuture,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::random::xoroshiro128::Xoroshiro;
use pumpkin_util::random::{RandomImpl, get_seed};
use pumpkin_world::world::SimpleWorld;
use std::any::Any;
use std::pin::Pin;
use std::sync::Arc;

use crate::world::World;

use super::BlockEntity;

pub struct EnderChestBlockEntity {
    pub position: BlockPos,

    // Viewer
    viewers: Arc<ViewerCountTracker>,
}

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

    fn write_nbt<'a>(
        &'a self,
        _nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async {})
    }

    fn tick<'a>(&'a self, world: Arc<World>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            self.viewers
                .update_viewer_count::<Self>(self, world, &self.position)
                .await;
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl ViewerCountListener for EnderChestBlockEntity {
    fn on_container_open<'a>(
        &'a self,
        world: &'a Arc<dyn SimpleWorld>,
        _position: &'a BlockPos,
    ) -> ViewerFuture<'a, ()> {
        Box::pin(async move {
            let world = Arc::downcast::<World>(world.clone().as_any()).unwrap();
            self.play_sound(&world, Sound::BlockEnderChestOpen).await;
        })
    }

    fn on_container_close<'a>(
        &'a self,
        world: &'a Arc<dyn SimpleWorld>,
        _position: &'a BlockPos,
    ) -> ViewerFuture<'a, ()> {
        Box::pin(async move {
            let world = Arc::downcast::<World>(world.clone().as_any()).unwrap();
            self.play_sound(&world, Sound::BlockEnderChestClose).await;
        })
    }

    fn on_viewer_count_update<'a>(
        &'a self,
        world: &'a Arc<dyn SimpleWorld>,
        position: &'a BlockPos,
        _old: u16,
        new: u16,
    ) -> ViewerFuture<'a, ()> {
        Box::pin(async move {
            world
                .add_synced_block_event(*position, Self::LID_ANIMATION_EVENT_TYPE, new as u8)
                .await;
        })
    }
}

impl EnderChestBlockEntity {
    pub const LID_ANIMATION_EVENT_TYPE: u8 = 1;
    pub const ID: &'static str = "minecraft:ender_chest";

    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            viewers: Arc::new(ViewerCountTracker::new()),
        }
    }

    #[must_use]
    pub fn get_tracker(&self) -> Arc<ViewerCountTracker> {
        self.viewers.clone()
    }

    async fn play_sound(&self, world: &Arc<World>, sound: Sound) {
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
