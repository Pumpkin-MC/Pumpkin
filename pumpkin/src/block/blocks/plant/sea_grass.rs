use async_trait::async_trait;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::java::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{BlockStateId, world::BlockAccessor};
use std::sync::Arc;

use crate::{
    block::{blocks::plant::PlantBlockBase, pumpkin_block::PumpkinBlock},
    entity::player::Player,
    server::Server,
    world::World,
};

#[pumpkin_block("minecraft:seagrass")]
pub struct SeaGrassBlock;

#[async_trait]
impl PumpkinBlock for SeaGrassBlock {
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
        <Self as PlantBlockBase>::can_place_at(self, block_accessor, block_pos).await
    }

    async fn get_state_for_neighbor_update(
        &self,
        world: &Arc<World>,
        _block: &Block,
        state: BlockStateId,
        pos: &BlockPos,
        _direction: BlockDirection,
        _neighbor_pos: &BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        <Self as PlantBlockBase>::get_state_for_neighbor_update(self, world.as_ref(), pos, state)
            .await
    }
}

impl PlantBlockBase for SeaGrassBlock {
    async fn can_plant_on_top(
        &self,
        block_accessor: &dyn pumpkin_world::world::BlockAccessor,
        pos: &pumpkin_util::math::position::BlockPos,
    ) -> bool {
        let block = block_accessor.get_block(pos).await;
        let block_state = block_accessor.get_block_state(pos).await;
        block_state.is_side_solid(BlockDirection::Up) && block != Block::MAGMA_BLOCK
    }
}
