use crate::block::{BlockBehaviour, BlockMetadata, OnPlaceArgs};
use pumpkin_data::{
    BlockId, BlockStateId,
    block_properties::{BlockProperties, SculkCatalystLikeProperties},
};

pub struct SculkCatalystBlock;

impl BlockMetadata for SculkCatalystBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::SCULK_CATALYST].into()
    }
}

impl BlockBehaviour for SculkCatalystBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = SculkCatalystLikeProperties::default(args.block);
        props.bloom = false;
        props.to_state_id(args.block)
    }
}
