use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, tag};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::BlockStateId;

use crate::block::{
    BlockBehaviour, BlockFuture, BrokenArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
};

use super::FireBlockBase;

#[pumpkin_block("minecraft:soul_fire")]
pub struct SoulFireBlock;

impl SoulFireBlock {
    #[must_use]
    pub fn is_soul_base(block: &Block) -> bool {
        block.has_tag(&tag::Block::MINECRAFT_SOUL_FIRE_BASE_BLOCKS)
    }
}

impl BlockBehaviour for SoulFireBlock {
    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if !Self::is_soul_base(args.world.get_block(&args.position.down()).await) {
                return Block::AIR.default_state.id;
            }

            args.state_id
        })
    }

    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            Self::is_soul_base(args.block_accessor.get_block(&args.position.down()).await)
        })
    }

    fn broken<'a>(&'a self, args: BrokenArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            FireBlockBase::broken(args.world, *args.position).await;
        })
    }
}
