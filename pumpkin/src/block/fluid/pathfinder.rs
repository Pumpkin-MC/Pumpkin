use crate::block::fluid::flowing_trait::FlowingFluid;
use crate::world::World;
use pumpkin_data::{
    Block, BlockDirection,
    fluid::{EnumVariants, Falling, Fluid, FluidProperties, Level},
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use std::sync::Arc;
type FlowingFluidProperties = pumpkin_data::fluid::FlowingWaterLikeFluidProperties;
use super::physics;

/// Represents a node in the BFS pathfinding queue for fluid flow calculation.
#[derive(Clone, Copy)]
pub struct PathNode {
    pub pos: BlockPos,
    pub distance: i32,
    pub exclude_dir: BlockDirection,
}

/// Checks if a position has a hole (downward flow opportunity) below it.
async fn is_hole(world: &Arc<World>, fluid: &Fluid, pos: &BlockPos) -> bool {
    let below_pos = pos.down();
    let below_state = world.get_block_state(&below_pos).await;
    let below_block = Block::from_state_id(below_state.id);
    physics::can_be_replaced(below_state, below_block, fluid)
}

/// Determines valid spread directions for fluid flow using hole-first priority.
///
/// - Holes (downward flow opportunities) get distance 0 priority
/// - All directions with equal minimum distance are returned
/// - Returns up to 4 directions with their computed fluid states
pub async fn get_spread<T: FlowingFluid + Sync + ?Sized>(
    fluid_impl: &T,
    world: &Arc<World>,
    fluid: &Fluid,
    block_pos: &BlockPos,
) -> ([(BlockDirection, BlockStateId); 4], usize) {
    let mut min_dist = 1000;
    let mut result = [(BlockDirection::North, BlockStateId::default()); 4];
    let mut result_count = 0;

    for direction in [
        BlockDirection::North,
        BlockDirection::South,
        BlockDirection::West,
        BlockDirection::East,
    ] {
        let side_pos = block_pos.offset(direction.to_offset());
        let side_state = world.get_block_state(&side_pos).await;
        let side_state_id = side_state.id;
        let side_block = Block::from_state_id(side_state.id);

        let side_props = FlowingFluidProperties::from_state_id(side_state_id, fluid);

        // Check if we can pass through (not a solid source block)
        if !physics::can_be_replaced(side_state, side_block, fluid)
            || (side_props.level == Level::L8 && side_props.falling != Falling::True)
        {
            continue;
        }

        let new_fluid_state = fluid_impl.get_new_liquid(world, fluid, &side_pos).await;

        // Skip if no valid fluid state for this position
        if new_fluid_state.is_none() {
            continue;
        }

        let new_fluid_props = new_fluid_state.unwrap();
        let new_state_id = new_fluid_props.to_state_id(fluid);

        // Holes get distance 0
        let slope_dist = if is_hole(world, fluid, &side_pos).await {
            0
        } else {
            get_in_flow_down_distance_iterative(
                fluid_impl,
                world,
                fluid,
                side_pos,
                direction.opposite(),
            )
            .await
        };

        // Clear results if we find a shorter path
        if slope_dist < min_dist {
            result_count = 0;
        }

        // Add all directions with equal minimum distance
        if slope_dist <= min_dist {
            // Check if the fluid at this position can be replaced
            let can_replace = if fluid_impl.is_same_fluid(fluid, side_state_id) {
                // Can replace if new level is higher or if target is falling
                let target_level = i32::from(side_props.level.to_index()) + 1;
                let new_level = i32::from(new_fluid_props.level.to_index()) + 1;
                new_level > target_level || side_props.falling == Falling::True
            } else {
                // Can replace non-fluid blocks (already checked in can_be_replaced)
                true
            };

            if can_replace && result_count < 4 {
                result[result_count] = (direction, new_state_id);
                result_count += 1;
            }

            min_dist = slope_dist;
        }
    }
    (result, result_count)
}

/// Performs iterative BFS search to find the shortest distance to a downward flow opportunity.
///
/// Uses stack-allocated array for zero heap allocations. Searches up to `get_max_flow_distance`
/// (dynamic) horizontally from the starting position.
///
/// # Returns
/// Distance to nearest hole, or 1000 if no hole found within search distance
pub async fn get_in_flow_down_distance_iterative<T: FlowingFluid + Sync + ?Sized>(
    fluid_impl: &T,
    world: &Arc<World>,
    fluid: &Fluid,
    start_pos: BlockPos,
    initial_exclude_dir: BlockDirection,
) -> i32 {
    const MAX_QUEUE_SIZE: usize = 64;

    let mut queue: [PathNode; MAX_QUEUE_SIZE] = [PathNode {
        pos: BlockPos::new(0, 0, 0),
        distance: 0,
        exclude_dir: BlockDirection::North,
    }; MAX_QUEUE_SIZE];

    let mut queue_start = 0;
    let mut queue_end = 0;

    queue[queue_end] = PathNode {
        pos: start_pos,
        distance: 1,
        exclude_dir: initial_exclude_dir,
    };
    queue_end = 1;

    let mut visited_bitset = [0u64; 4];
    let slope_find_distance = fluid_impl.get_max_flow_distance(world);

    let get_bit_index = |pos: BlockPos| -> Option<usize> {
        let dx = pos.0.x - start_pos.0.x + slope_find_distance;
        let dz = pos.0.z - start_pos.0.z + slope_find_distance;
        let grid_size = slope_find_distance * 2 + 1;
        (dx >= 0 && dx < grid_size && dz >= 0 && dz < grid_size)
            .then(|| (dz * grid_size + dx) as usize)
    };

    while queue_start < queue_end {
        let node = queue[queue_start];
        queue_start += 1;

        if node.distance > slope_find_distance {
            continue;
        }

        if let Some(bit_idx) = get_bit_index(node.pos) {
            let word_idx = bit_idx / 64;
            let bit_pos = bit_idx % 64;
            if (visited_bitset[word_idx] & (1u64 << bit_pos)) != 0 {
                continue;
            }
            visited_bitset[word_idx] |= 1u64 << bit_pos;
        }

        // Check for hole (downward flow opportunity)
        let below_pos = node.pos.down();
        let below_state = world.get_block_state(&below_pos).await;
        let below_block = Block::from_state_id(below_state.id);
        if physics::can_be_replaced(below_state, below_block, fluid) {
            return node.distance;
        }

        for direction in [
            BlockDirection::North,
            BlockDirection::South,
            BlockDirection::West,
            BlockDirection::East,
        ] {
            if direction == node.exclude_dir {
                continue;
            }

            let next_pos = node.pos.offset(direction.to_offset());

            let next_state = world.get_block_state(&next_pos).await;
            let next_block = Block::from_state_id(next_state.id);
            if !physics::can_be_replaced(next_state, next_block, fluid) {
                continue;
            }

            // Source blocks block horizontal pathfinding
            let next_state_id = world.get_block_state_id(&next_pos).await;
            if fluid_impl.is_same_fluid(fluid, next_state_id) {
                let next_props = FlowingFluidProperties::from_state_id(next_state_id, fluid);
                if next_props.level == Level::L8 && next_props.falling == Falling::False {
                    continue;
                }
            }

            if queue_end < MAX_QUEUE_SIZE {
                queue[queue_end] = PathNode {
                    pos: next_pos,
                    distance: node.distance + 1,
                    exclude_dir: direction.opposite(),
                };
                queue_end += 1;
            }
        }
    }

    1000
}
