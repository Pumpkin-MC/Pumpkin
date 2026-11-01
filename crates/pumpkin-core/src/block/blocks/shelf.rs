use pumpkin_data::HorizontalFacingExt;
use pumpkin_data::block_properties::AcaciaShelfLikeProperties;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::{BlockState, BlockStateId};
use pumpkin_inventory::Inventory;
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_world::tick::TickPriority;

use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::entities::shelf::ShelfBlockEntity;
use crate::block::{
    BlockBehaviour, GetComparatorOutputArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs,
    PathComputationType, PlacedArgs,
};
use crate::entity::EntityBase;
use std::sync::Arc;

#[pumpkin_block_from_tag("minecraft:wooden_shelves")]
pub struct ShelfBlock;

impl BlockBehaviour for ShelfBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut properties = AcaciaShelfLikeProperties::default(args.block);

        properties.facing = args.player.get_entity().get_horizontal_facing().opposite();
        properties.powered = block_receives_redstone_power(args.world, args.position);
        properties.waterlogged = args.replacing.water_source();

        properties.to_state_id(args.block)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let props = AcaciaShelfLikeProperties::from_state_id(args.state_id);
        if props.waterlogged {
            args.world.schedule_fluid_tick(
                &Fluid::WATER,
                *args.position,
                Fluid::WATER.flow_speed as u8,
                TickPriority::Normal,
            );
        }
        args.state_id
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        {
            let entity = ShelfBlockEntity::new(*args.position);
            args.world.add_block_entity(Arc::new(entity));
        }
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        // Vanilla reads a shelf only through its back face.
        let props = AcaciaShelfLikeProperties::from_state_id(args.state.id);
        if args.direction != props.facing.to_block_direction().opposite() {
            return Some(0);
        }

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
