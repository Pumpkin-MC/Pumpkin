use crate::block::{BlockBehaviour, BlockMetadata, OnPlaceArgs, PathComputationType};
use pumpkin_data::block_properties::MangroveRootsLikeProperties;
use pumpkin_data::{Block, BlockId, BlockState, BlockStateId};

pub struct MangroveRootsBlock;

impl BlockMetadata for MangroveRootsBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::MANGROVE_ROOTS, BlockId::HEAVY_CORE].into()
    }
}

impl BlockBehaviour for MangroveRootsBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = MangroveRootsLikeProperties::default(args.block);
        props.waterlogged = args.replacing.water_source();
        props.to_state_id(args.block)
    }

    fn is_pathfindable(&self, state: &BlockState, _computation_type: PathComputationType) -> bool {
        Block::from_state_id(state.id) != &Block::HEAVY_CORE
    }
}
