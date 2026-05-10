use crate::block::{BlockBehaviour, BlockFuture, OnPlaceArgs};
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_macros::pumpkin_block;
use pumpkin_world::BlockStateId;

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
}
