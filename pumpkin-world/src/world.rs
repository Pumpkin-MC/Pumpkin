use std::sync::Arc;

use crate::block::entities::BlockEntity;
use crate::{BlockStateId, inventory::Inventory};
use async_trait::async_trait;
use bitflags::bitflags;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{Block, BlockDirection, BlockState};
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

#[async_trait]
pub trait SimpleWorld: BlockAccessor + Send + Sync {
    async fn set_block_state(
        self: Arc<Self>,
        position: &BlockPos,
        block_state_id: BlockStateId,
        flags: BlockFlags,
    ) -> BlockStateId;

    async fn update_neighbor(
        self: Arc<Self>,
        neighbor_block_pos: &BlockPos,
        source_block: &pumpkin_data::Block,
    );

    async fn update_neighbors(
        self: Arc<Self>,
        block_pos: &BlockPos,
        except: Option<BlockDirection>,
    );

    async fn remove_block_entity(&self, block_pos: &BlockPos);
    async fn get_block_entity(&self, block_pos: &BlockPos) -> Option<Arc<dyn BlockEntity>>;
    async fn get_world_age(&self) -> i64;

    async fn play_sound(&self, sound: Sound, category: SoundCategory, position: &Vector3<f64>);
    async fn play_sound_fine(
        &self,
        sound: Sound,
        category: SoundCategory,
        position: &Vector3<f64>,
        volume: f32,
        pitch: f32,
    );

    /* ItemScatterer */
    async fn scatter_inventory(
        self: Arc<Self>,
        position: &BlockPos,
        inventory: &Arc<dyn Inventory>,
    );
}

#[async_trait]
pub trait BlockRegistryExt: Send + Sync {
    fn can_place_at(
        &self,
        block: &Block,
        block_accessor: &dyn BlockAccessor,
        block_pos: &BlockPos,
        face: BlockDirection,
    ) -> bool;
}

#[async_trait]
pub trait BlockAccessor: Send + Sync {
    async fn get_block(&self, position: &BlockPos) -> &'static Block;

    async fn get_block_state(&self, position: &BlockPos) -> &'static BlockState;

    async fn get_block_and_state(
        &self,
        position: &BlockPos,
    ) -> (&'static Block, &'static BlockState);
}
