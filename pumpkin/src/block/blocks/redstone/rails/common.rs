use std::sync::Arc;

use pumpkin_data::{
    Block,
    block_properties::{HorizontalFacing, RailShape, StraightRailShape},
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{
    BlockStateId,
    world::{BlockAccessor, BlockFlags},
};

use crate::world::World;

use super::{HorizontalFacingRailExt, Rail, RailElevation, RailProperties, StraightRailShapeExt};

pub(super) async fn rail_placement_is_valid(world: &World, block: &Block, pos: &BlockPos) -> bool {
    if !can_place_rail_at(world, pos).await {
        return false;
    }

    let state_id = world.get_block_state_id(pos).await;
    let rail_props = RailProperties::new(state_id, block);
    let rail_leaning_direction = match rail_props.shape() {
        RailShape::AscendingNorth => Some(HorizontalFacing::North),
        RailShape::AscendingSouth => Some(HorizontalFacing::South),
        RailShape::AscendingEast => Some(HorizontalFacing::East),
        RailShape::AscendingWest => Some(HorizontalFacing::West),
        _ => None,
    };

    if let Some(direction) = rail_leaning_direction {
        if !can_place_rail_at(world, &pos.offset(direction.to_offset()).up()).await {
            return false;
        }
    }

    true
}

pub(super) async fn can_place_rail_at(world: &dyn BlockAccessor, pos: &BlockPos) -> bool {
    let state = world.get_block_state(&pos.down()).await;
    state.is_solid()
}

pub(super) async fn compute_placed_rail_shape(
    world: &World,
    block_pos: &BlockPos,
    player_facing: HorizontalFacing,
) -> StraightRailShape {
    // Use the same sophisticated logic as normal rails, but adapted for straight rails
    // Check each direction for rail connections, similar to normal rail placement

    // Check East first
    if let Some(east_rail) = Rail::find_if_unlocked(world, block_pos, HorizontalFacing::East).await
    {
        // Check for opposite connection (West) to form a straight line
        if let Some(west_rail) =
            Rail::find_if_unlocked(world, block_pos, HorizontalFacing::West).await
        {
            // We have connections in both East and West
            if east_rail.elevation == RailElevation::Up {
                return StraightRailShape::AscendingEast;
            } else if west_rail.elevation == RailElevation::Up {
                return StraightRailShape::AscendingWest;
            }
            return StraightRailShape::EastWest;
        }
        // Only East connection
        if east_rail.elevation == RailElevation::Up {
            return StraightRailShape::AscendingEast;
        }
        return StraightRailShape::EastWest;
    }

    // Check South
    if let Some(south_rail) =
        Rail::find_if_unlocked(world, block_pos, HorizontalFacing::South).await
    {
        // Check for opposite connection (North) to form a straight line
        if let Some(north_rail) =
            Rail::find_if_unlocked(world, block_pos, HorizontalFacing::North).await
        {
            // We have connections in both South and North
            if south_rail.elevation == RailElevation::Up {
                return StraightRailShape::AscendingSouth;
            } else if north_rail.elevation == RailElevation::Up {
                return StraightRailShape::AscendingNorth;
            }
            return StraightRailShape::NorthSouth;
        }
        // Only South connection
        if south_rail.elevation == RailElevation::Up {
            return StraightRailShape::AscendingSouth;
        }
        return StraightRailShape::NorthSouth;
    }

    // Check West
    if let Some(west_rail) = Rail::find_if_unlocked(world, block_pos, HorizontalFacing::West).await
    {
        if west_rail.elevation == RailElevation::Up {
            return StraightRailShape::AscendingWest;
        }
        return StraightRailShape::EastWest;
    }

    // Check North
    if let Some(north_rail) =
        Rail::find_if_unlocked(world, block_pos, HorizontalFacing::North).await
    {
        if north_rail.elevation == RailElevation::Up {
            return StraightRailShape::AscendingNorth;
        }
        return StraightRailShape::NorthSouth;
    }

    // No connections found, use player facing direction
    player_facing.to_rail_shape_flat()
}

pub(super) async fn update_flanking_rails_shape(
    world: &Arc<World>,
    block: &Block,
    state_id: BlockStateId,
    block_pos: &BlockPos,
) {
    for direction in RailProperties::new(state_id, block).directions() {
        let Some(mut flanking_rail) =
            Rail::find_with_elevation(world, block_pos.offset(direction.to_offset())).await
        else {
            // Skip non-rail blocks
            continue;
        };

        let new_shape =
            compute_flanking_rail_new_shape(world, &flanking_rail, direction.opposite()).await;

        if new_shape != flanking_rail.properties.shape() {
            flanking_rail.properties.set_shape(new_shape);
            world
                .set_block_state(
                    &flanking_rail.position,
                    flanking_rail.properties.to_state_id(flanking_rail.block),
                    BlockFlags::NOTIFY_ALL,
                )
                .await;
        }
    }
}

async fn compute_flanking_rail_new_shape(
    world: &World,
    rail: &Rail,
    flanking_from: HorizontalFacing,
) -> RailShape {
    let mut connected_towards = Vec::with_capacity(2);
    let mut is_already_connected_to_elevated_rail = false;

    for neighbor_direction in rail.properties.directions() {
        if neighbor_direction == flanking_from {
            // Rails pointing to where the player placed are not connected
            continue;
        }

        let Some(maybe_connected_rail) =
            Rail::find_with_elevation(world, rail.position.offset(neighbor_direction.to_offset()))
                .await
        else {
            // Rails pointing to non-rail blocks are not connected
            continue;
        };

        if maybe_connected_rail
            .properties
            .directions()
            .into_iter()
            .any(|d| d == neighbor_direction.opposite())
        {
            // Rails pointing to other rails that are pointing back are connected
            connected_towards.push(neighbor_direction);
            is_already_connected_to_elevated_rail =
                maybe_connected_rail.elevation == RailElevation::Up;
        }
    }

    let new_neighbor_directions = match connected_towards.len() {
        2 => {
            // Do not update rails that are locked (aka fully connected)
            return rail.properties.shape();
        }
        1 => [connected_towards[0], flanking_from],
        0 => [flanking_from, flanking_from.opposite()],
        _ => unreachable!("Rails only have two sides"),
    };

    // Handle rails that want to be straight
    if new_neighbor_directions
        .iter()
        .all(|d| *d == flanking_from || *d == flanking_from.opposite())
    {
        if rail.elevation == RailElevation::Down {
            if is_already_connected_to_elevated_rail {
                // Prioritize the South/West ascending
                return match flanking_from {
                    HorizontalFacing::South | HorizontalFacing::North => RailShape::AscendingSouth,
                    HorizontalFacing::West | HorizontalFacing::East => RailShape::AscendingWest,
                };
            }

            return flanking_from.to_rail_shape_ascending_towards().as_shape();
        } else if is_already_connected_to_elevated_rail {
            return connected_towards[0]
                .to_rail_shape_ascending_towards()
                .as_shape();
        }

        // Reset the shape to flat even if the rail already had good directions
        return rail.get_new_rail_shape(new_neighbor_directions[0], new_neighbor_directions[1]);
    }

    // Handle straight rails that want to curve
    if !rail.properties.can_curve() {
        return if new_neighbor_directions[0] == HorizontalFacing::North
            || new_neighbor_directions[0] == HorizontalFacing::South
        {
            if rail.elevation == RailElevation::Down {
                // The rail is down so it should be ascending
                flanking_from.to_rail_shape_ascending_towards().as_shape()
            } else {
                rail.get_new_rail_shape(new_neighbor_directions[0], new_neighbor_directions[1])
            }
        } else {
            rail.properties.shape()
        };
    }

    rail.get_new_rail_shape(new_neighbor_directions[0], new_neighbor_directions[1])
}
