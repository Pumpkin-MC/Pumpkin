use pumpkin_data::tag::Taggable;
use pumpkin_data::{BlockId, BlockStateId, tag};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockAccessor;

use crate::block::blocks::plant::PlantBlockBase;
use crate::block::blocks::plant::sapling::SaplingBlock;
use crate::block::{
    BlockBehaviour, BlockMetadata, BonemealArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    PathComputationType,
};

pub struct AzaleaBlock;

impl BlockMetadata for AzaleaBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::AZALEA, BlockId::FLOWERING_AZALEA].into()
    }
}

impl BlockBehaviour for AzaleaBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        <Self as PlantBlockBase>::get_state_for_neighbor_update(
            self,
            args.world,
            args.position,
            args.state_id,
        )
    }

    fn is_valid_bonemeal_target(&self, _args: BonemealArgs<'_>) -> bool {
        true
    }

    fn is_bonemeal_success(&self, _args: BonemealArgs<'_>) -> bool {
        rand::random::<f32>() < 0.45
    }

    fn perform_bonemeal(&self, args: BonemealArgs<'_>) {
        SaplingBlock::advance_tree(args.world, args.position, args.block, args.state_id, true);
    }

    fn is_pathfindable(
        &self,
        _state: &pumpkin_data::BlockState,
        _computation_type: PathComputationType,
    ) -> bool {
        false
    }
}

impl PlantBlockBase for AzaleaBlock {
    fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let block_below = block_accessor.get_block(pos);
        block_below.has_tag(&tag::Block::MINECRAFT_SUPPORTS_AZALEA)
            || block_below.has_tag(&tag::Block::MINECRAFT_SUPPORTS_VEGETATION)
    }

    fn can_place_at(&self, block_accessor: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
        self.can_plant_on_top(block_accessor, &block_pos.down())
    }
}
