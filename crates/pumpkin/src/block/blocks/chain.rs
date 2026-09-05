use crate::block::{BlockBehaviour, OnPlaceArgs, PathComputationType};
use pumpkin_data::block_properties::Axis;
use pumpkin_data::{BlockDirection, BlockState, BlockStateId};
use pumpkin_macros::pumpkin_block;

#[pumpkin_block("minecraft:iron_chain")]
pub struct ChainBlock;

impl BlockBehaviour for ChainBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
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

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}
