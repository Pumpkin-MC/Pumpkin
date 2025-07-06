use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::java::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{
    BlockStateId,
    world::{BlockAccessor, BlockFlags},
};

use crate::{
    block::{blocks::plant::PlantBlockBase, pumpkin_block::PumpkinBlock},
    entity::{EntityBase, player::Player},
    server::Server,
    world::World,
};
use pumpkin_world::world::BlockFlags;

use crate::block::pumpkin_block::{CanPlaceAtArgs, OnEntityCollisionArgs, PumpkinBlock};

#[pumpkin_block("minecraft:lily_pad")]
pub struct LilyPadBlock;

#[async_trait]
impl PumpkinBlock for LilyPadBlock {
    async fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        // Proberbly not the best solution, but works
        if args
            .entity
            .get_entity()
            .entity_type
            .resource_name
            .ends_with("_boat")
        {
            args.world
                .break_block(args.position, None, BlockFlags::empty())
                .await;
        }
    }

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

impl PlantBlockBase for LilyPadBlock {
    async fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let block = block_accessor.get_block(pos).await;
        let above_fluid = block_accessor.get_block(&pos.up()).await;
        (block == &Block::WATER || block == &Block::ICE)
            && (above_fluid != &Block::WATER && above_fluid != &Block::LAVA)
    }
}
