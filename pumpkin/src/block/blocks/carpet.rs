use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::tag::{RegistryKey, get_tag_values};
use pumpkin_data::{Block, BlockDirection};
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::chunk::TickPriority;
use pumpkin_world::world::{BlockAccessor, BlockFlags};
use std::sync::Arc;

pub struct CarpetBlock;

impl BlockMetadata for CarpetBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        get_tag_values(RegistryKey::Block, "minecraft:wool_carpets").unwrap()
    }
}

#[async_trait]
impl PumpkinBlock for CarpetBlock {
    async fn can_place_at(
        &self,
        _server: Option<&Server>,
        _world: Option<&World>,
        block_accessor: &dyn BlockAccessor,
        _player: Option<&Player>,
        _block: &Block,
        block_pos: &BlockPos,
        _face: BlockDirection,
        _use_item_on: Option<&SUseItemOn>,
    ) -> bool {
        can_place_at(block_accessor, block_pos).await
    }

    async fn get_state_for_neighbor_update(
        &self,
        world: &World,
        block: &Block,
        state: BlockStateId,
        pos: &BlockPos,
        _direction: BlockDirection,
        _neighbor_pos: &BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        if !can_place_at(world, pos).await {
            world
                .schedule_block_tick(block, *pos, 1, TickPriority::Normal)
                .await;
        }
        state
    }

    async fn on_scheduled_tick(&self, world: &Arc<World>, _block: &Block, pos: &BlockPos) {
        if !can_place_at(world.as_ref(), pos).await {
            world.break_block(pos, None, BlockFlags::empty()).await;
        }
    }
}

#[pumpkin_block("minecraft:moss_carpet")]
pub struct MossCarpetBlock;

#[async_trait]
impl PumpkinBlock for MossCarpetBlock {
    async fn can_place_at(
        &self,
        _server: Option<&Server>,
        _world: Option<&World>,
        block_accessor: &dyn BlockAccessor,
        _player: Option<&Player>,
        _block: &Block,
        block_pos: &BlockPos,
        _face: BlockDirection,
        _use_item_on: Option<&SUseItemOn>,
    ) -> bool {
        can_place_at(block_accessor, block_pos).await
    }

    async fn get_state_for_neighbor_update(
        &self,
        world: &World,
        block: &Block,
        state: BlockStateId,
        pos: &BlockPos,
        _direction: BlockDirection,
        _neighbor_pos: &BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        if !can_place_at(world, pos).await {
            world
                .schedule_block_tick(block, *pos, 1, TickPriority::Normal)
                .await;
        }
        state
    }

    async fn on_scheduled_tick(&self, world: &Arc<World>, _block: &Block, pos: &BlockPos) {
        if !can_place_at(world.as_ref(), pos).await {
            world.break_block(pos, None, BlockFlags::empty()).await;
        }
    }
}

#[pumpkin_block("minecraft:pale_moss_carpet")]
pub struct PaleMossCarpetBlock;

#[async_trait]
impl PumpkinBlock for PaleMossCarpetBlock {
    async fn can_place_at(
        &self,
        _server: Option<&Server>,
        _world: Option<&World>,
        block_accessor: &dyn BlockAccessor,
        _player: Option<&Player>,
        _block: &Block,
        block_pos: &BlockPos,
        _face: BlockDirection,
        _use_item_on: Option<&SUseItemOn>,
    ) -> bool {
        can_place_at(block_accessor, block_pos).await
    }

    async fn get_state_for_neighbor_update(
        &self,
        world: &World,
        block: &Block,
        state: BlockStateId,
        pos: &BlockPos,
        _direction: BlockDirection,
        _neighbor_pos: &BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        if !can_place_at(world, pos).await {
            world
                .schedule_block_tick(block, *pos, 1, TickPriority::Normal)
                .await;
        }
        state
    }

    async fn on_scheduled_tick(&self, world: &Arc<World>, _block: &Block, pos: &BlockPos) {
        if !can_place_at(world.as_ref(), pos).await {
            world.break_block(pos, None, BlockFlags::empty()).await;
        }
    }
}

async fn can_place_at(block_accessor: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
    !block_accessor
        .get_block_state(&block_pos.down())
        .await
        .is_air()
}
