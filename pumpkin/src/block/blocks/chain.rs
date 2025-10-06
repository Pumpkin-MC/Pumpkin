use crate::block::{BlockBehaviour, OnPlaceArgs};
use async_trait::async_trait;
use pumpkin_data::BlockDirection;
use pumpkin_data::block_properties::Axis;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_world::BlockStateId;
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_data::tag::get_tag_values;
use pumpkin_data::tag::RegistryKey;

#[pumpkin_block_from_tag("minecraft:chains")]
pub struct ChainBlock;

#[async_trait]
impl BlockBehaviour for ChainBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props =
            pumpkin_data::block_properties::IronChainLikeProperties::default(args.block);
        props.r#waterlogged = args.replacing.water_source();
        props.r#axis = match args.direction {
            BlockDirection::East | BlockDirection::West => Axis::X,
            BlockDirection::Up | BlockDirection::Down => Axis::Y,
            BlockDirection::North | BlockDirection::South => Axis::Z,
        };

        props.to_state_id(args.block)
    }
}
