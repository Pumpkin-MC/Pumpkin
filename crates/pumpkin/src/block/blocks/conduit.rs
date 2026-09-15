use crate::block::entities::conduit::ConduitBlockEntity;
use crate::block::{BlockBehaviour, OnPlaceArgs, PathComputationType, PlacedArgs};
use pumpkin_data::{BlockState, BlockStateId};
use pumpkin_macros::pumpkin_block;
use std::sync::Arc;

#[pumpkin_block("minecraft:conduit")]
pub struct ConduitBlock;

impl BlockBehaviour for ConduitBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props =
            pumpkin_data::block_properties::MangroveRootsLikeProperties::default(args.block);
        props.r#waterlogged = args.replacing.water_source();

        props.to_state_id(args.block)
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        {
            let entity = ConduitBlockEntity::new(*args.position);
            args.world.add_block_entity(Arc::new(entity));
        }
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}
