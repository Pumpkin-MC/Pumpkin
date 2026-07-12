use crate::block::entities::conduit::ConduitBlockEntity;
use crate::block::{BlockBehaviour, BlockFuture, OnPlaceArgs, PlacedArgs};
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_macros::pumpkin_block;
use std::sync::Arc;

#[pumpkin_block("minecraft:conduit")]
pub struct ConduitBlock;

impl BlockBehaviour for ConduitBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props =
                pumpkin_data::block_properties::MangroveRootsLikeProperties::default(args.block);
            props.r#waterlogged = args.replacing.water_source();

            props.to_state_id(args.block)
        })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let entity = ConduitBlockEntity::new(*args.position);
            args.world.add_block_entity(Arc::new(entity));
        })
    }
}
