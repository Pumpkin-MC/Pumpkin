use pumpkin_data::block_properties::AcaciaShelfLikeProperties;
use pumpkin_data::{BlockState, BlockStateId};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_world::inventory::Inventory;

use crate::block::entities::shelf::ShelfBlockEntity;
use crate::block::{
    BlockBehaviour, GetComparatorOutputArgs, OnPlaceArgs, PathComputationType, PlacedArgs,
};
use crate::entity::EntityBase;
use std::sync::Arc;

#[pumpkin_block_from_tag("minecraft:wooden_shelves")]
pub struct ShelfBlock;

impl BlockBehaviour for ShelfBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut properties = AcaciaShelfLikeProperties::default(args.block);

        properties.facing = args.player.get_entity().get_horizontal_facing().opposite();

        properties.to_state_id(args.block)
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        {
            let entity = ShelfBlockEntity::new(*args.position);
            args.world.add_block_entity(Arc::new(entity));
        }
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        if let Some(block_entity) = args.world.get_block_entity(args.position)
            && let Some(shelf) = block_entity.as_any().downcast_ref::<ShelfBlockEntity>()
        {
            let b0 = u8::from(!shelf.get_stack(0).is_empty());
            let b1 = u8::from(!shelf.get_stack(1).is_empty());
            let b2 = u8::from(!shelf.get_stack(2).is_empty());
            Some(b0 | (b1 << 1) | (b2 << 2))
        } else {
            None
        }
    }

    fn is_pathfindable(&self, state: &BlockState, computation_type: PathComputationType) -> bool {
        computation_type == PathComputationType::Water && state.is_waterlogged()
    }
}
