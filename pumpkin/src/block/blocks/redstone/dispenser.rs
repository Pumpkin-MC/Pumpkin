use crate::block::{BlockBehaviour, BlockFuture, OnPlaceArgs};
use pumpkin_data::block_properties::{BlockProperties, DispenserLikeProperties};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::BlockStateId;

#[pumpkin_block("minecraft:dispenser")]
pub struct DispenserBlock;

impl BlockBehaviour for DispenserBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = DispenserLikeProperties::default(args.block);
            props.facing = args.player.living_entity.entity.get_facing().opposite();
            props.to_state_id(args.block)
        })
    }
}
