use crate::block::{BlockBehaviour, BlockFuture, OnPlaceArgs};
use pumpkin_data::block_properties::{BlockProperties, LecternLikeProperties};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::BlockStateId;

#[pumpkin_block("minecraft:lectern")]
pub struct LecternBlock;

impl BlockBehaviour for LecternBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = LecternLikeProperties::default(args.block);
            props.facing = args
                .player
                .living_entity
                .entity
                .get_horizontal_facing()
                .opposite();
            props.to_state_id(args.block)
        })
    }
}
