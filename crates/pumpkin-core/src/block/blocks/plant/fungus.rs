use crate::block::GetStateForNeighborUpdateArgs;
use crate::block::{BlockBehaviour, BlockMetadata, CanPlaceAtArgs};
use pumpkin_data::BlockStateId;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockId, tag};

pub struct FungusBlock;

impl BlockMetadata for FungusBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::CRIMSON_FUNGUS, BlockId::WARPED_FUNGUS].into()
    }
}

impl BlockBehaviour for FungusBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        let block_below = args.block_accessor.get_block(&args.position.down());
        if args.block == &Block::WARPED_FUNGUS {
            block_below.has_tag(&tag::Block::MINECRAFT_SUPPORTS_WARPED_FUNGUS)
        } else {
            block_below.has_tag(&tag::Block::MINECRAFT_SUPPORTS_CRIMSON_FUNGUS)
        }
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let block_below = args.world.get_block(&args.position.down());
        let can_survive = if args.block == &Block::WARPED_FUNGUS {
            block_below.has_tag(&tag::Block::MINECRAFT_SUPPORTS_WARPED_FUNGUS)
        } else {
            block_below.has_tag(&tag::Block::MINECRAFT_SUPPORTS_CRIMSON_FUNGUS)
        };
        if !can_survive {
            return Block::AIR.default_state.id;
        }
        args.state_id
    }
}
