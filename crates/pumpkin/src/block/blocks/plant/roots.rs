use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockId, BlockStateId, tag};

use crate::block::{BlockBehaviour, BlockMetadata, CanPlaceAtArgs, GetStateForNeighborUpdateArgs};

pub struct RootsBlock;

impl BlockMetadata for RootsBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::WARPED_ROOTS, BlockId::CRIMSON_ROOTS].into()
    }
}

impl BlockBehaviour for RootsBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        let block_below = args.block_accessor.get_block(&args.position.down());
        if args.block == &Block::WARPED_ROOTS {
            block_below.has_tag(&tag::Block::MINECRAFT_SUPPORTS_WARPED_ROOTS)
        } else {
            block_below.has_tag(&tag::Block::MINECRAFT_SUPPORTS_CRIMSON_ROOTS)
        }
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let block_below = args.world.get_block(&args.position.down());
        let can_survive = if args.block == &Block::WARPED_ROOTS {
            block_below.has_tag(&tag::Block::MINECRAFT_SUPPORTS_WARPED_ROOTS)
        } else {
            block_below.has_tag(&tag::Block::MINECRAFT_SUPPORTS_CRIMSON_ROOTS)
        };
        if !can_survive {
            return Block::AIR.default_state.id;
        }
        args.state_id
    }
}
