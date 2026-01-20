use std::pin::Pin;
use std::sync::Arc;

use crate::block::entities::BlockEntity;
use crate::{BlockStateId, inventory::Inventory};
use bitflags::bitflags;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{Block, BlockDirection, BlockState};
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use thiserror::Error;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct BlockFlags: u32 {
        const NOTIFY_NEIGHBORS                      = 0b000_0000_0001;
        const NOTIFY_LISTENERS                      = 0b000_0000_0010;
        const NOTIFY_ALL                            = 0b000_0000_0011;
        const FORCE_STATE                           = 0b000_0000_0100;
        const SKIP_DROPS                            = 0b000_0000_1000;
        const MOVED                                 = 0b000_0001_0000;
        const SKIP_REDSTONE_WIRE_STATE_REPLACEMENT  = 0b000_0010_0000;
        const SKIP_BLOCK_ENTITY_REPLACED_CALLBACK   = 0b000_0100_0000;
        const SKIP_BLOCK_ADDED_CALLBACK             = 0b000_1000_0000;
    }
}

#[derive(Debug, Error)]
pub enum GetBlockError {
    InvalidBlockId,
    BlockOutOfWorldBounds,
}

impl std::fmt::Display for GetBlockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

pub type WorldFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait SimpleWorld: BlockAccessor + Send + Sync {
    fn set_block_state<'a>(
        self: Arc<Self>,
        position: &'a BlockPos,
        block_state_id: BlockStateId,
        flags: BlockFlags,
    ) -> WorldFuture<'a, BlockStateId>;

    fn update_neighbor<'a>(
        self: Arc<Self>,
        neighbor_block_pos: &'a BlockPos,
        source_block: &'a pumpkin_data::Block,
    ) -> WorldFuture<'a, ()>;

    fn update_neighbors<'a>(
        self: Arc<Self>,
        block_pos: &'a BlockPos,
        except: Option<BlockDirection>,
    ) -> WorldFuture<'a, ()>;

    fn is_space_empty<'a>(&'a self, bounding_box: BoundingBox) -> WorldFuture<'a, bool>;

    fn spawn_from_type(
        self: Arc<Self>,
        entity_type: &'static EntityType,
        position: Vector3<f64>,
    ) -> WorldFuture<'static, ()>;

    fn add_synced_block_event<'a>(
        &'a self,
        pos: BlockPos,
        r#type: u8,
        data: u8,
    ) -> WorldFuture<'a, ()>;

    fn sync_world_event<'a>(
        &'a self,
        world_event: WorldEvent,
        position: BlockPos,
        data: i32,
    ) -> WorldFuture<'a, ()>;

    fn remove_block_entity<'a>(&'a self, block_pos: &'a BlockPos) -> WorldFuture<'a, ()>;

    fn get_block_entity<'a>(
        &'a self,
        block_pos: &'a BlockPos,
    ) -> WorldFuture<'a, Option<Arc<dyn BlockEntity>>>;

    fn get_world_age<'a>(&'a self) -> WorldFuture<'a, i64>;

    fn play_sound<'a>(
        &'a self,
        sound: Sound,
        category: SoundCategory,
        position: &'a Vector3<f64>,
    ) -> WorldFuture<'a, ()>;

    fn play_sound_fine<'a>(
        &'a self,
        sound: Sound,
        category: SoundCategory,
        position: &'a Vector3<f64>,
        volume: f32,
        pitch: f32,
    ) -> WorldFuture<'a, ()>;

    /* ItemScatterer */
    fn scatter_inventory<'a>(
        self: Arc<Self>,
        position: &'a BlockPos,
        inventory: &'a Arc<dyn Inventory>,
    ) -> WorldFuture<'a, ()>;
}

pub trait BlockRegistryExt: Send + Sync {
    fn can_place_at(
        &self,
        block: &Block,
        state: &BlockState,
        block_accessor: &dyn BlockAccessor,
        block_pos: &BlockPos,
    ) -> bool;
}

pub trait BlockAccessor: Send + Sync {
    fn get_block<'a>(
        &'a self,
        position: &'a BlockPos,
    ) -> Pin<Box<dyn Future<Output = &'static Block> + Send + 'a>>;

    fn get_block_state<'a>(
        &'a self,
        position: &'a BlockPos,
    ) -> Pin<Box<dyn Future<Output = &'static BlockState> + Send + 'a>>;

    fn get_block_state_id<'a>(
        &'a self,
        position: &'a BlockPos,
    ) -> Pin<Box<dyn Future<Output = BlockStateId> + Send + 'a>>;

    fn get_block_and_state<'a>(
        &'a self,
        position: &'a BlockPos,
    ) -> Pin<Box<dyn Future<Output = (&'static Block, &'static BlockState)> + Send + 'a>>;
}
