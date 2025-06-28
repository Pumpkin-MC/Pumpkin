use async_trait::async_trait;
use pumpkin_data::tag::Tagable;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::BlockAccessor;
use std::sync::Arc;

use crate::block::blocks::plant::PlantBlockBase;
use crate::block::pumpkin_block::PumpkinBlock;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;

#[pumpkin_block("minecraft:nether_wart")]
pub struct NetherWartBlock;

#[async_trait]
impl PumpkinBlock for NetherWartBlock {
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

impl PlantBlockBase for NetherWartBlock {
    async fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let block = block_accessor.get_block(pos).await;
        block.is_tagged_with("minecraft:soul_sand").unwrap()
    }
}
