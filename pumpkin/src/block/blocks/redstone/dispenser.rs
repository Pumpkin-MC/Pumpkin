use crate::block::entities::dispenser::DispenserBlockEntity;
use crate::block::{BlockBehaviour, BlockFuture, OnPlaceArgs, PlacedArgs};
use crate::entity::EntityBase;
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::{BlockProperties, DispenserLikeProperties};
use pumpkin_macros::pumpkin_block;
use std::sync::Arc;

#[pumpkin_block("minecraft:dispenser")]
pub struct DispenserBlock;

impl BlockBehaviour for DispenserBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = DispenserLikeProperties::default(args.block);
            props.facing = args.player.get_entity().get_facing().opposite();
            props.to_state_id(args.block)
        })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let dispenser_block_entity = DispenserBlockEntity::new(*args.position);
            args.world
                .add_block_entity(Arc::new(dispenser_block_entity));
        })
    }
}
