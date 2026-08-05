use crate::block::{
    BlockBehaviour, BlockFuture, BlockMetadata, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
};
use pumpkin_data::BlockStateId;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockId, tag};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockAccessor;
pub struct FungusBlock;

impl BlockMetadata for FungusBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::CRIMSON_FUNGUS, BlockId::WARPED_FUNGUS].into()
    }
}

/// Vanilla gives every fungus its own support tag, chosen from the fungus being placed
/// and not from the block it is standing on.
fn has_support(block_accessor: &dyn BlockAccessor, fungus: &Block, position: &BlockPos) -> bool {
    let ground = block_accessor.get_block(&position.down());
    if fungus == &Block::WARPED_FUNGUS {
        ground.has_tag(&tag::Block::MINECRAFT_SUPPORTS_WARPED_FUNGUS)
    } else {
        ground.has_tag(&tag::Block::MINECRAFT_SUPPORTS_CRIMSON_FUNGUS)
    }
}

impl BlockBehaviour for FungusBlock {
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
