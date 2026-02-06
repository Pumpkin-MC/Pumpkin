use pumpkin_data::block_properties::{AcaciaShelfLikeProperties, BlockProperties};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_world::BlockStateId;

use crate::block::{BlockBehaviour, BlockFuture, OnPlaceArgs};
use crate::entity::EntityBase;

#[pumpkin_block_from_tag("minecraft:wooden_shelves")]
pub struct ShelfBlock;

impl BlockBehaviour for ShelfBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut properties = AcaciaShelfLikeProperties::default(args.block);

            // Face in the opposite direction the player is facing
            properties.facing = args.player.get_entity().get_horizontal_facing().opposite();

            properties.to_state_id(args.block)
        })
    }
}
