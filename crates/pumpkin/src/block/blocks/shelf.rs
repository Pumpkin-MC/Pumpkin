use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::{AcaciaShelfLikeProperties, BlockProperties};
use pumpkin_macros::pumpkin_block_from_tag;

use crate::block::entities::shelf::ShelfBlockEntity;
use crate::block::{BlockBehaviour, BlockFuture, OnPlaceArgs, PlacedArgs};
use crate::entity::EntityBase;
use std::sync::Arc;

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

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let entity = ShelfBlockEntity::new(*args.position);
            args.world.add_block_entity(Arc::new(entity));
        })
    }
}
