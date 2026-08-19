use crate::block::{BlockBehaviour, BlockFuture, BlockMetadata, GetComparatorOutputArgs};
use pumpkin_data::BlockId;
use pumpkin_data::block_properties::{BeeNestLikeProperties, BlockProperties};

pub struct BeehiveBlock;

impl BlockMetadata for BeehiveBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::BEEHIVE, BlockId::BEE_NEST].into()
    }
}

impl BlockBehaviour for BeehiveBlock {
    fn get_comparator_output<'a>(
        &'a self,
        args: GetComparatorOutputArgs<'a>,
    ) -> BlockFuture<'a, Option<u8>> {
        Box::pin(async move {
            let state_id = args.world.get_block_state_id(args.position);
            let props = BeeNestLikeProperties::from_state_id(state_id, args.block);
            Some(props.honey_level)
        })
    }
}
