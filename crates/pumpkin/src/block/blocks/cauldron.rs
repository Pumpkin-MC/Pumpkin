use crate::block::{BlockBehaviour, BlockFuture, BlockMetadata, GetComparatorOutputArgs};
use pumpkin_data::BlockId;
use pumpkin_data::block_properties::{BlockProperties, WaterCauldronLikeProperties};

pub struct CauldronBlock;

impl BlockMetadata for CauldronBlock {
    fn ids() -> Box<[BlockId]> {
        [
            BlockId::CAULDRON,
            BlockId::WATER_CAULDRON,
            BlockId::LAVA_CAULDRON,
            BlockId::POWDER_SNOW_CAULDRON,
        ]
        .into()
    }
}

impl BlockBehaviour for CauldronBlock {
    fn get_comparator_output<'a>(
        &'a self,
        args: GetComparatorOutputArgs<'a>,
    ) -> BlockFuture<'a, Option<u8>> {
        Box::pin(async move {
            match args.block.id {
                BlockId::WATER_CAULDRON | BlockId::POWDER_SNOW_CAULDRON => {
                    let state_id = args.world.get_block_state_id(args.position);
                    let props = WaterCauldronLikeProperties::from_state_id(state_id, args.block);
                    Some(props.level)
                }
                BlockId::LAVA_CAULDRON => Some(3),
                _ => Some(0),
            }
        })
    }
}
