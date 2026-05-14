use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, tag};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{BlockStateId, world::BlockAccessor};

use crate::block::{
    BlockBehaviour, BlockFuture, BlockMetadata, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    blocks::plant::PlantBlockBase,
};

pub struct DryVegetationBlock;

impl BlockMetadata for DryVegetationBlock {
    fn ids() -> Box<[u16]> {
        [
            Block::DEAD_BUSH.id,
            Block::TALL_DRY_GRASS.id,
            Block::SHORT_DRY_GRASS.id,
        ]
        .into()
    }
}

impl BlockBehaviour for DryVegetationBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position)
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            <Self as PlantBlockBase>::get_state_for_neighbor_update(
                self,
                args.world,
                args.position,
                args.state_id,
            )
            .await
        })
    }
}

impl PlantBlockBase for DryVegetationBlock {
    fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
        let block_below = block_accessor.get_block(block_pos);
        block_below.has_tag(&tag::Block::MINECRAFT_SUPPORTS_DRY_VEGETATION)
    }
}
