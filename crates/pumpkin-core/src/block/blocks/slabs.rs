use pumpkin_data::block_properties::SlabType;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::{BlockDirection, BlockState, BlockStateId};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_world::tick::TickPriority;

use crate::block::{
    BlockBehaviour, BlockIsReplacing, CanUpdateAtArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs,
    PathComputationType,
};

type SlabProperties = pumpkin_data::block_properties::WhiteWoolSlabLikeProperties;

#[pumpkin_block_from_tag("minecraft:slabs")]
pub struct SlabBlock;

impl BlockBehaviour for SlabBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        if let BlockIsReplacing::Itself(state_id) = args.replacing {
            let mut slab_props = SlabProperties::from_state_id(state_id);
            slab_props.r#type = SlabType::Double;
            slab_props.waterlogged = false;
            return slab_props.to_state_id(args.block);
        }

        let mut slab_props = SlabProperties::default(args.block);
        slab_props.waterlogged = args.replacing.water_source();
        slab_props.r#type = match args.direction {
            BlockDirection::Up => SlabType::Top,
            BlockDirection::Down => SlabType::Bottom,
            _ => match args.use_item_on.cursor_pos.y {
                0.0..=0.5 => SlabType::Bottom,
                _ => SlabType::Top,
            },
        };

        slab_props.to_state_id(args.block)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let slab_props = SlabProperties::from_state_id(args.state_id);
        if slab_props.waterlogged {
            args.world.schedule_fluid_tick(
                &Fluid::WATER,
                *args.position,
                Fluid::WATER.flow_speed as u8,
                TickPriority::Normal,
            );
        }
        args.state_id
    }

    fn can_update_at(&self, args: CanUpdateAtArgs<'_>) -> bool {
        let slab_props = SlabProperties::from_state_id(args.state_id);

        slab_props.r#type
            == match args.direction {
                BlockDirection::Up => SlabType::Bottom,
                BlockDirection::Down => SlabType::Top,
                _ => match args.use_item_on.cursor_pos.y {
                    0.0..=0.5 => SlabType::Top,
                    _ => SlabType::Bottom,
                },
            }
    }

    fn is_pathfindable(&self, state: &BlockState, computation_type: PathComputationType) -> bool {
        match computation_type {
            PathComputationType::Water => state.is_waterlogged(),
            PathComputationType::Land | PathComputationType::Air => false,
        }
    }
}
