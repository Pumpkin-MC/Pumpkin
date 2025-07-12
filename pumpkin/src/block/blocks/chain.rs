use crate::block::pumpkin_block::{OnPlaceArgs, PumpkinBlock};
use async_trait::async_trait;
use pumpkin_data::BlockDirection;
use pumpkin_data::block_properties::Axis;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_macros::pumpkin_block;
use pumpkin_world::BlockStateId;

#[pumpkin_block("minecraft:chain")]
pub struct ChainBlock;

#[async_trait]
impl PumpkinBlock for ChainBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = pumpkin_data::block_properties::ChainLikeProperties::default(args.block);
        props.r#waterlogged = args.replacing.water_source();
        props.r#axis = match args.direction {
            BlockDirection::East | BlockDirection::West => Axis::X,
            BlockDirection::Up | BlockDirection::Down => Axis::Y,
            BlockDirection::North | BlockDirection::South => Axis::Z,
        };

        props.to_state_id(args.block)
    }
}
