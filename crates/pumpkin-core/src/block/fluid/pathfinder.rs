use super::physics;
use crate::block::fluid::flowing_trait::FlowingFluid;
use crate::world::World;
use pumpkin_data::{
    Block, BlockDirection,
    fluid::{Falling, FlowingFluidProperties, Fluid, FluidFlow, Level},
};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

/// Checks if a position has a hole (downward flow opportunity) below it.
fn is_hole(world: &Arc<World>, fluid: &'static Fluid, pos: &BlockPos) -> bool {
    let below_pos = pos.down();
    let below_state = world.get_block_state(&below_pos);
    let below_block = Block::from_state_id(below_state.id);
    physics::can_be_replaced(below_state, below_block, fluid)
}

/// Determines valid spread directions for fluid flow using hole-first priority.
///
/// - Holes (downward flow opportunities) get distance 0 priority
/// - All directions with equal minimum distance are returned
/// - One entry per horizontal direction that receives fluid
pub fn get_spread<T: FlowingFluid + Sync + ?Sized>(
    fluid_impl: &T,
    world: &Arc<World>,
    fluid: &'static Fluid,
    block_pos: &BlockPos,
) -> [Option<(BlockDirection, FlowingFluidProperties)>; 4] {
    let mut min_dist = 1000;
    let mut result = [None; 4];

    for (slot, direction) in [
        BlockDirection::North,
        BlockDirection::South,
        BlockDirection::West,
        BlockDirection::East,
    ]
    .into_iter()
    .enumerate()
    {
        let side_pos = block_pos.offset(direction.to_offset());
        let side_state = world.get_block_state(&side_pos);
        let side_state_id = side_state.id;
        let side_block = Block::from_state_id(side_state.id);

        let side_fluid_props = fluid_impl.get_effective_props(fluid, side_state_id);

        // Check if we can pass through (not a solid source block or waterlogged)
        if !physics::can_be_replaced(side_state, side_block, fluid)
            || side_fluid_props
                .as_ref()
                .is_some_and(|p| p.level == Level::L8 && p.falling != Falling::True)
        {
            continue;
        }

        // Skip if no valid fluid state for this position
        let Some(new_fluid_props) = fluid_impl.get_new_liquid(world, fluid, &side_pos) else {
            continue;
        };

        // Holes get distance 0
        let slope_dist = if is_hole(world, fluid, &side_pos) {
            0
        } else {
            get_in_flow_down_distance_iterative(
                fluid_impl,
                world,
                fluid,
                side_pos,
                direction.opposite(),
            )
        };

        // Clear results if we find a shorter path
        if slope_dist < min_dist {
            result = [None; 4];
        }

        // Add all directions with equal minimum distance
        if slope_dist <= min_dist {
            // Check if the fluid at this position can be replaced
            let can_replace = side_fluid_props.as_ref().is_none_or(|sp| {
                // Can replace if new level is higher or if target is falling
                let target_level = i32::from(sp.level.to_index()) + 1;
                let new_level = i32::from(new_fluid_props.level.to_index()) + 1;
                new_level > target_level || sp.falling == Falling::True
            });

            if can_replace {
                result[slot] = Some((direction, new_fluid_props));
            }

            min_dist = slope_dist;
        }
    }
    result
}

/// Largest hole search reach: `getSlopeFindDistance` + 1.
const MAX_REACH: i32 = FluidFlow::MAX_SLOPE_FIND_DISTANCE as i32 + 1;
/// Grid around the start holding every node within reach.
const GRID: i32 = 2 * MAX_REACH + 1;
const CELLS: usize = (GRID * GRID) as usize;

/// BFS distance to the nearest hole, 1000 if none is within reach.
///
/// Each grid cell is queued at most once, so the queue cannot overflow.
pub fn get_in_flow_down_distance_iterative<T: FlowingFluid + Sync + ?Sized>(
    fluid_impl: &T,
    world: &Arc<World>,
    fluid: &'static Fluid,
    start_pos: BlockPos,
    initial_exclude_dir: BlockDirection,
) -> i32 {
    let reach = fluid_impl.get_max_flow_distance(world);
    let cell = |pos: BlockPos| {
        ((pos.0.z - start_pos.0.z + MAX_REACH) * GRID + pos.0.x - start_pos.0.x + MAX_REACH)
            as usize
    };

    let mut visited = [false; CELLS];
    // the spreading block itself is never part of a path
    visited[cell(start_pos.offset(initial_exclude_dir.to_offset()))] = true;
    visited[cell(start_pos)] = true;
    let mut queue = [(start_pos, 0); CELLS];
    queue[0] = (start_pos, 1);
    let (mut head, mut tail) = (0, 1);

    while head < tail {
        let (pos, distance) = queue[head];
        head += 1;

        let below_state = world.get_block_state(&pos.down());
        let below_block = Block::from_state_id(below_state.id);
        if physics::can_be_replaced(below_state, below_block, fluid) {
            return distance;
        }
        if distance == reach {
            continue;
        }

        for direction in [
            BlockDirection::North,
            BlockDirection::South,
            BlockDirection::West,
            BlockDirection::East,
        ] {
            let next_pos = pos.offset(direction.to_offset());
            if visited[cell(next_pos)] {
                continue;
            }

            let next_state = world.get_block_state(&next_pos);
            let next_block = Block::from_state_id(next_state.id);
            if !physics::can_be_replaced(next_state, next_block, fluid) {
                continue;
            }

            // Source blocks (including waterlogged) block horizontal pathfinding
            if fluid_impl
                .get_effective_props(fluid, next_state.id)
                .is_some_and(|p| p.level == Level::L8 && p.falling == Falling::False)
            {
                continue;
            }

            visited[cell(next_pos)] = true;
            queue[tail] = (next_pos, distance + 1);
            tail += 1;
        }
    }

    1000
}
