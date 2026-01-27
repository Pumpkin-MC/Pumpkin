use pumpkin_data::{
    block_properties::{AcaciaShelfLikeProperties, BlockProperties},
    tag,
};
use pumpkin_world::BlockStateId;

use crate::block::{BlockBehaviour, BlockFuture, BlockMetadata, OnPlaceArgs};
use crate::entity::EntityBase;

pub struct ShelfBlock;

impl BlockMetadata for ShelfBlock {
    fn ids() -> Box<[u16]> {
        tag::Block::MINECRAFT_WOODEN_SHELVES.1.into()
    }
}

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
