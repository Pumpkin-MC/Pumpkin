use crate::block::{BlockBehaviour, BlockFuture, BlockMetadata, OnPlaceArgs};
use pumpkin_data::block_properties::{BlockProperties, WallTorchLikeProperties};
use pumpkin_data::tag::{self};
use pumpkin_world::BlockStateId;

pub struct GlazedTerracottaBlock;
impl BlockMetadata for GlazedTerracottaBlock {
    fn ids() -> Box<[u16]> {
        tag::Item::C_GLAZED_TERRACOTTAS.1.into()
    }
}

impl BlockBehaviour for GlazedTerracottaBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut prop = WallTorchLikeProperties::default(args.block);
            prop.facing = args
                .player
                .living_entity
                .entity
                .get_horizontal_facing()
                .opposite();
            prop.to_state_id(args.block)
        })
    }
}
