use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockId, BlockStateId, tag};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockAccessor;

use crate::block::{
    BlockBehaviour, BlockFuture, BlockMetadata, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
};

pub struct RootsBlock;

impl BlockMetadata for RootsBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::WARPED_ROOTS, BlockId::CRIMSON_ROOTS].into()
    }
}

/// Vanilla gives every roots block its own support tag, chosen from the roots being placed
/// and not from the block they are standing on.
fn has_support(block_accessor: &dyn BlockAccessor, roots: &Block, position: &BlockPos) -> bool {
    let ground = block_accessor.get_block(&position.down());
    if roots == &Block::WARPED_ROOTS {
        ground.has_tag(&tag::Block::MINECRAFT_SUPPORTS_WARPED_ROOTS)
    } else {
        ground.has_tag(&tag::Block::MINECRAFT_SUPPORTS_CRIMSON_ROOTS)
    }
}

impl BlockBehaviour for RootsBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        has_support(args.block_accessor, args.block, args.position)
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if has_support(args.world, args.block, args.position) {
                args.state_id
            } else {
                Block::AIR.default_state.id
            }
        })
    }
}
