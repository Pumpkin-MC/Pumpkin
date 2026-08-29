use crate::block::{BlockBehaviour, BlockMetadata, GetComparatorOutputArgs};
use pumpkin_data::BlockId;
use pumpkin_data::block_properties::{BeeNestLikeProperties, BlockProperties};

pub struct BeehiveBlock;

impl BlockMetadata for BeehiveBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::BEEHIVE, BlockId::BEE_NEST].into()
    }
}

impl BlockBehaviour for BeehiveBlock {
    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        {
            let state_id = args.world.get_block_state_id(args.position);
            let props = BeeNestLikeProperties::from_state_id(state_id, args.block);
            Some(props.honey_level)
        }
    }
}
