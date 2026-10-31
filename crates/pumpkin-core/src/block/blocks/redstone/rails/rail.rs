use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::HorizontalFacing;
use pumpkin_data::block_properties::RailShape;
use pumpkin_data::fluid::Fluid;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

use crate::block::BlockBehaviour;
use crate::block::CanPlaceAtArgs;
use crate::block::GetStateForNeighborUpdateArgs;
use crate::block::OnNeighborUpdateArgs;
use crate::block::OnPlaceArgs;
use crate::block::OnStateReplacedArgs;
use crate::block::PlacedArgs;
use crate::world::World;

use super::super::block_receives_redstone_power;
use super::StraightRailShapeExt;
use super::common::{can_place_rail_at, rail_placement_is_valid, update_flanking_rails_shape};
use super::{HorizontalFacingRailExt, Rail, RailElevation, RailProperties};

#[pumpkin_block("minecraft:rail")]
pub struct RailBlock;

fn compute_curved_rail_shape(
    world: &World,
    block_pos: &BlockPos,
    default_flat: RailShape,
    has_signal: bool,
) -> RailShape {
    let north = Rail::find_if_unlocked(world, block_pos, HorizontalFacing::North);
    let south = Rail::find_if_unlocked(world, block_pos, HorizontalFacing::South);
    let west = Rail::find_if_unlocked(world, block_pos, HorizontalFacing::West);
    let east = Rail::find_if_unlocked(world, block_pos, HorizontalFacing::East);

    let n = north.is_some();
    let s = south.is_some();
    let w = west.is_some();
    let e = east.is_some();

    let north_or_south = n || s;
    let west_or_east = w || e;

    let mut shape = if north_or_south && !west_or_east {
        Some(RailShape::NorthSouth)
    } else if west_or_east && !north_or_south {
        Some(RailShape::EastWest)
    } else {
        None
    };

    let south_and_east = s && e;
    let south_and_west = s && w;
    let north_and_east = n && e;
    let north_and_west = n && w;

    if south_and_east && !n && !w {
        shape = Some(RailShape::SouthEast);
    }
    if south_and_west && !n && !e {
        shape = Some(RailShape::SouthWest);
    }
    if north_and_west && !s && !e {
        shape = Some(RailShape::NorthWest);
    }
    if north_and_east && !s && !w {
        shape = Some(RailShape::NorthEast);
    }

    if shape.is_none() {
        if north_or_south && west_or_east {
            shape = Some(default_flat);
        } else if north_or_south {
            shape = Some(RailShape::NorthSouth);
        } else if west_or_east {
            shape = Some(RailShape::EastWest);
        }

        if has_signal {
            if south_and_east {
                shape = Some(RailShape::SouthEast);
            }
            if south_and_west {
                shape = Some(RailShape::SouthWest);
            }
            if north_and_east {
                shape = Some(RailShape::NorthEast);
            }
            if north_and_west {
                shape = Some(RailShape::NorthWest);
            }
        } else {
            if north_and_west {
                shape = Some(RailShape::NorthWest);
            }
            if north_and_east {
                shape = Some(RailShape::NorthEast);
            }
            if south_and_west {
                shape = Some(RailShape::SouthWest);
            }
            if south_and_east {
                shape = Some(RailShape::SouthEast);
            }
        }
    }

    let mut final_shape = shape.unwrap_or(default_flat);
    if final_shape == RailShape::NorthSouth {
        if north.is_some_and(|r| r.elevation == RailElevation::Up) {
            final_shape = RailShape::AscendingNorth;
        }
        if south.is_some_and(|r| r.elevation == RailElevation::Up) {
            final_shape = RailShape::AscendingSouth;
        }
    }
    if final_shape == RailShape::EastWest {
        if east.is_some_and(|r| r.elevation == RailElevation::Up) {
            final_shape = RailShape::AscendingEast;
        }
        if west.is_some_and(|r| r.elevation == RailElevation::Up) {
            final_shape = RailShape::AscendingWest;
        }
    }
    final_shape
}

fn count_potential_connections(world: &World, pos: &BlockPos) -> usize {
    [
        HorizontalFacing::North,
        HorizontalFacing::South,
        HorizontalFacing::West,
        HorizontalFacing::East,
    ]
    .into_iter()
    .filter(|dir| Rail::find_with_elevation(world, pos.offset(dir.to_offset())).is_some())
    .count()
}

impl BlockBehaviour for RailBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut rail_props = RailProperties::default(args.block);
        rail_props.set_waterlogged(args.replacing.water_source());

        let default_flat = args
            .player
            .living_entity
            .entity
            .get_horizontal_facing()
            .to_rail_shape_flat()
            .as_shape();
        let has_signal = block_receives_redstone_power(args.world, args.position);
        let shape = compute_curved_rail_shape(args.world, args.position, default_flat, has_signal);

        rail_props.set_shape(shape);
        rail_props.to_state_id(args.block)
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        update_flanking_rails_shape(args.world, args.block, args.state_id, args.position);
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if RailProperties::new(args.state_id, args.block).is_waterlogged() {
            args.world.schedule_fluid_tick(
                &Fluid::WATER,
                *args.position,
                Fluid::WATER.flow_speed as u8,
                TickPriority::Normal,
            );
        }
        args.state_id
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        if !rail_placement_is_valid(args.world, args.block, args.position) {
            args.world
                .break_block(args.position, None, BlockFlags::NOTIFY_ALL);
            return;
        }

        if count_potential_connections(args.world, args.position) == 3 {
            let state_id = args.world.get_block_state_id(args.position);
            let mut rail_props = RailProperties::new(state_id, args.block);
            let has_signal = block_receives_redstone_power(args.world, args.position);
            let new_shape = compute_curved_rail_shape(
                args.world,
                args.position,
                rail_props.shape(),
                has_signal,
            );
            if new_shape != rail_props.shape() {
                rail_props.set_shape(new_shape);
                let new_state_id = rail_props.to_state_id(args.block);
                args.world
                    .set_block_state(args.position, new_state_id, BlockFlags::NOTIFY_ALL);
                update_flanking_rails_shape(args.world, args.block, new_state_id, args.position);
            }
        }
    }

    fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        if !args.moved {
            let rail_props = RailProperties::new(args.old_state_id, args.block);
            if rail_props.shape().is_ascending() {
                args.world
                    .update_neighbors_at(&args.position.up(), args.block, None);
            }
        }
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        can_place_rail_at(args.block_accessor, args.position)
    }
}
