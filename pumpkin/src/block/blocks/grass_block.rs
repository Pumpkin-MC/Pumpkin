use pumpkin_data::{
    Block,
    block_properties::{BlockProperties, GrassBlockLikeProperties},
    tag::{self, Taggable},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::BlockStateId;

use crate::block::{BlockBehaviour, BlockFuture, GetStateForNeighborUpdateArgs};

#[pumpkin_block("minecraft:grass_block")]
pub struct GrassBlock;

impl BlockBehaviour for GrassBlock {
    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let block_above = args.world.get_block(&args.position.up());
            let mut props =
                GrassBlockLikeProperties::from_state_id(args.state_id, &Block::GRASS_BLOCK);
            let should_be_snowy = block_above.has_tag(&tag::Block::MINECRAFT_SNOW);
            if props.snowy == should_be_snowy {
                return args.state_id;
            }
            props.snowy = should_be_snowy;

            props.to_state_id(&Block::GRASS_BLOCK)
        })
    }
}
