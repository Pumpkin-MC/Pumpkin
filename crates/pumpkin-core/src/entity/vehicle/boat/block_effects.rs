use std::collections::HashSet;

use pumpkin_data::Block;
use pumpkin_util::math::{
    boundingbox::BoundingBox,
    position::BlockPos,
    vector3::{Axis, Vector3},
};

use super::BoatEntity;

const BLOCK_INTERSECTION_EPSILON: f32 = 1.0e-5;
const EPSILON: f64 = 1.0e-7;
const MAX_MOVEMENTS_HANDELED_PER_TICK: usize = 100;
const MAX_BLOCK_ITERATIONS_ALONG_TRAVEL_PER_TICK: usize = 16;

pub(super) struct BoatMovement {
    from: Vector3<f64>,
    to: Vector3<f64>,
    requested: Option<Vector3<f64>>,
}

impl BoatEntity {
    pub(crate) fn record_movement(
        &self,
        from: Vector3<f64>,
        to: Vector3<f64>,
        requested_movement: Vector3<f64>,
    ) {
        let mut movements = self
            .movements
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // Entity.addMovementThisTick merges the oldest pair into a straight movement.
        if movements.len() >= MAX_MOVEMENTS_HANDELED_PER_TICK
            && let Some(first) = movements.pop_front()
            && let Some(second) = movements.front_mut()
        {
            second.from = first.from;
            second.requested = None;
        }
        movements.push_back(BoatMovement {
            from,
            to,
            requested: Some(requested_movement),
        });
    }

    pub(super) fn apply_effects_from_blocks(&self) {
        let mut movements = std::mem::take(
            &mut *self
                .movements
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        let entity = &self.vehicle.entity;
        if !entity.is_affected_by_blocks() {
            return;
        }
        let to = entity.pos.load();
        let from = movements
            .back()
            .map_or_else(|| entity.last_pos.load(), |movement| movement.to);
        if movements.is_empty()
            || (to - from).length_squared()
                > f64::from(BLOCK_INTERSECTION_EPSILON * BLOCK_INTERSECTION_EPSILON)
        {
            movements.push_back(BoatMovement {
                from,
                to,
                requested: None,
            });
        }
        let world = entity.world.load();
        let Some(server) = world.server.upgrade() else {
            return;
        };
        let mut visited = HashSet::new();
        let mut visit = |pos| {
            if !entity.is_alive() {
                return;
            }
            let (block, state) = world.get_block_and_state(&pos);
            // Lily pads inherit the full-block entity-inside shape, not their thin outline.
            if block == &Block::LILY_PAD && visited.insert(pos) {
                world
                    .block_registry
                    .on_entity_collision(block, &world, self, &pos, state, &server);
            }
        };
        for movement in movements {
            let bounds = entity
                .bounding_box
                .load()
                .shift(movement.from - entity.pos.load());
            for_each_block_along_movement(
                bounds,
                movement.to - movement.from,
                movement.requested,
                &mut visit,
            );
        }
    }
}

fn for_each_block_along_movement(
    bounds: BoundingBox,
    movement: Vector3<f64>,
    requested_movement: Option<Vector3<f64>>,
    mut visit: impl FnMut(BlockPos),
) {
    // Entity.checkInsideBlocks follows the collision axes, with 16 steps per movement.
    let mut bounds = bounds;
    let destination = bounds
        .shift(movement)
        .expand_all(-f64::from(BLOCK_INTERSECTION_EPSILON));
    let mut remaining_steps = MAX_BLOCK_ITERATIONS_ALONG_TRAVEL_PER_TICK;
    let Some(requested_movement) = requested_movement.filter(|_| movement != Vector3::default())
    else {
        remaining_steps = visit_movement_blocks(bounds, movement, remaining_steps, &mut visit);
        if remaining_steps == 0 {
            for pos in BlockPos::iterate(
                destination.min_block_pos(),
                BlockPos::floored_v(destination.max),
            ) {
                visit(pos);
            }
        }
        return;
    };
    for axis in axis_step_order(requested_movement) {
        let distance = movement.get_axis(axis);
        if distance == 0.0 {
            continue;
        }
        let mut offset = Vector3::default();
        offset.set_axis(axis, distance);
        remaining_steps = visit_movement_blocks(bounds, offset, remaining_steps, &mut visit);
        if remaining_steps == 0 {
            break;
        }
        bounds = bounds.shift(offset);
    }
    if remaining_steps == 0 {
        for pos in BlockPos::iterate(
            destination.min_block_pos(),
            BlockPos::floored_v(destination.max),
        ) {
            visit(pos);
        }
    }
}

fn visit_movement_blocks(
    bounds: BoundingBox,
    movement: Vector3<f64>,
    remaining_steps: usize,
    visit: &mut impl FnMut(BlockPos),
) -> usize {
    let destination = bounds
        .shift(movement)
        .expand_all(-f64::from(BLOCK_INTERSECTION_EPSILON));
    if movement.length_squared()
        < f64::from(BLOCK_INTERSECTION_EPSILON * BLOCK_INTERSECTION_EPSILON)
    {
        for pos in BlockPos::iterate(
            destination.min_block_pos(),
            BlockPos::floored_v(destination.max),
        ) {
            visit(pos);
        }
        return remaining_steps - 1;
    }
    let mut visited = HashSet::new();
    let mut last_step = 0;
    let mut visit_step = |pos, iteration| {
        if !visited.insert(pos) {
            return true;
        }
        if iteration >= remaining_steps {
            return false;
        }
        last_step = iteration;
        visit(pos);
        true
    };
    let start = destination.shift(movement * -1.0);
    for pos in between_corners_in_direction(
        start.min_block_pos(),
        BlockPos::floored_v(start.max),
        movement,
    ) {
        visit_step(pos, 0);
    }
    if let Some(steps) = add_collisions_along_travel(destination, movement, &mut visit_step) {
        for pos in between_corners_in_direction(
            destination.min_block_pos(),
            BlockPos::floored_v(destination.max),
            movement,
        ) {
            if !visit_step(pos, steps + 1) {
                break;
            }
        }
    }
    remaining_steps - (last_step + 1)
}

fn add_collisions_along_travel(
    destination: BoundingBox,
    movement: Vector3<f64>,
    visit: &mut impl FnMut(BlockPos, usize) -> bool,
) -> Option<usize> {
    let corner = furthest_corner(movement);
    let size = destination.max - destination.min;
    let mut to = (destination.min + destination.max) * 0.5;
    for component in Axis::all() {
        to.set_axis(
            component,
            to.get_axis(component) + size.get_axis(component) * 0.5 * corner.get_axis(component),
        );
    }
    let from = to - movement;
    let mut cell = BlockPos::floored_v(from);
    let (step, interval, mut next) = traversal_steps(from, movement);
    let mut steps = 0;
    while next.x <= 1.0 || next.y <= 1.0 || next.z <= 1.0 {
        let axis = next_traversal_axis(next);
        cell.0
            .set_axis(axis, cell.0.get_axis(axis) + step.get_axis(axis));
        next.set_axis(axis, next.get_axis(axis) + interval.get_axis(axis));
        let Some(point) = clip_block(cell, from, to) else {
            continue;
        };
        steps += 1;
        let opposite = opposite_corner(point, cell, size, corner);
        for pos in between_corners_in_direction(cell, opposite, movement) {
            if !visit(pos, steps) {
                return None;
            }
        }
    }
    Some(steps)
}

fn furthest_corner(movement: Vector3<f64>) -> Vector3<f64> {
    let sign = |value| if value >= 0.0 { 1.0 } else { -1.0 };
    let (x, y, z) = (sign(movement.x), sign(movement.y), sign(movement.z));
    if movement.x.abs() <= movement.y.abs() && movement.x.abs() <= movement.z.abs() {
        Vector3::new(-x, -z, y)
    } else if movement.y.abs() <= movement.z.abs() {
        Vector3::new(z, -y, -x)
    } else {
        Vector3::new(-y, x, -z)
    }
}

fn traversal_steps(
    from: Vector3<f64>,
    movement: Vector3<f64>,
) -> (Vector3<i32>, Vector3<f64>, Vector3<f64>) {
    let mut steps = Vector3::default();
    let mut intervals = Vector3::default();
    let mut next = Vector3::default();
    for axis in Axis::all() {
        let distance = movement.get_axis(axis);
        let step = if distance == 0.0 {
            0
        } else if distance > 0.0 {
            1
        } else {
            -1
        };
        let interval = if step == 0 {
            f64::MAX
        } else {
            f64::from(step) / distance
        };
        let fraction = from.get_axis(axis) - from.get_axis(axis).floor();
        steps.set_axis(axis, step);
        intervals.set_axis(axis, interval);
        next.set_axis(
            axis,
            interval * if step > 0 { 1.0 - fraction } else { fraction },
        );
    }
    (steps, intervals, next)
}

fn next_traversal_axis(next: Vector3<f64>) -> Axis {
    if next.x < next.y {
        if next.x < next.z { Axis::X } else { Axis::Z }
    } else if next.y < next.z {
        Axis::Y
    } else {
        Axis::Z
    }
}

fn clip_block(cell: BlockPos, from: Vector3<f64>, to: Vector3<f64>) -> Option<Vector3<f64>> {
    let movement = to - from;
    let mut nearest = 1.0;
    for axis in [Axis::X, Axis::Y, Axis::Z] {
        let distance = movement.get_axis(axis);
        if distance.abs() <= EPSILON {
            continue;
        }
        let plane = f64::from(cell.0.get_axis(axis)) + if distance < 0.0 { 1.0 } else { 0.0 };
        let time = (plane - from.get_axis(axis)) / distance;
        // AABB.clip excludes hits exactly at the start or end of the segment.
        if time <= 0.0 || time >= nearest {
            continue;
        }
        let point = from + movement * time;
        if Axis::excluding(axis).iter().all(|&component| {
            let start = f64::from(cell.0.get_axis(component));
            point.get_axis(component) > start - EPSILON
                && point.get_axis(component) < start + 1.0 + EPSILON
        }) {
            nearest = time;
        }
    }
    (nearest < 1.0).then(|| from + movement * nearest)
}

fn opposite_corner(
    point: Vector3<f64>,
    cell: BlockPos,
    size: Vector3<f64>,
    corner: Vector3<f64>,
) -> BlockPos {
    let mut opposite = Vector3::default();
    let epsilon = f64::from(BLOCK_INTERSECTION_EPSILON);
    for axis in Axis::all() {
        let start = f64::from(cell.0.get_axis(axis));
        opposite.set_axis(
            axis,
            point
                .get_axis(axis)
                .clamp(start + epsilon, start + 1.0 - epsilon)
                - size.get_axis(axis) * corner.get_axis(axis),
        );
    }
    BlockPos::floored_v(opposite)
}

fn axis_step_order(movement: Vector3<f64>) -> [Axis; 3] {
    if movement.x.abs() < movement.z.abs() {
        [Axis::Y, Axis::Z, Axis::X]
    } else {
        Axis::all()
    }
}

fn between_corners_in_direction(
    first: BlockPos,
    second: BlockPos,
    movement: Vector3<f64>,
) -> impl Iterator<Item = BlockPos> {
    let mut min = Vector3::default();
    let mut max = Vector3::default();
    for axis in Axis::all() {
        min.set_axis(axis, first.0.get_axis(axis).min(second.0.get_axis(axis)));
        max.set_axis(axis, first.0.get_axis(axis).max(second.0.get_axis(axis)));
    }
    let [slow, middle, fast] = axis_step_order(movement);
    let size = max - min;
    let end = BlockPos::new(
        size.get_axis(fast),
        size.get_axis(middle),
        size.get_axis(slow),
    );
    BlockPos::iterate(BlockPos::ZERO, end).map(move |offset| {
        let mut pos = BlockPos::ZERO;
        for (axis, offset) in [(fast, offset.0.x), (middle, offset.0.y), (slow, offset.0.z)] {
            pos.0.set_axis(
                axis,
                if movement.get_axis(axis) < 0.0 {
                    max.get_axis(axis) - offset
                } else {
                    min.get_axis(axis) + offset
                },
            );
        }
        pos
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deflated_max_on_grid_boundary_keeps_vanilla_boundary_cell() {
        let bounds = BoundingBox::new(
            Vector3::new(0.25, 0.25, 0.25),
            Vector3::new(1.000_009_999_999_747_4, 0.75, 0.75),
        );
        let mut visited = HashSet::new();
        for_each_block_along_movement(bounds, Vector3::default(), None, |pos| {
            visited.insert(pos);
        });
        // Recorded from 26.3 BlockGetter with a deflated maximum X of exactly 1.0.
        assert_eq!(
            visited,
            HashSet::from([BlockPos::new(0, 0, 0), BlockPos::new(1, 0, 0)])
        );
    }

    #[test]
    fn fast_movement_visits_crossed_blocks_without_grazing_adjacent_blocks() {
        let bounds = BoundingBox::new(
            Vector3::new(0.25, 0.25, 0.25),
            Vector3::new(0.75, 0.75, 1.0),
        );
        let movement = Vector3::new(4.0, 0.0, 0.0);
        let mut visited = HashSet::new();
        for_each_block_along_movement(bounds, movement, Some(movement), |pos| {
            visited.insert(pos);
        });
        assert_eq!(
            visited,
            HashSet::from([
                BlockPos::new(0, 0, 0),
                BlockPos::new(1, 0, 0),
                BlockPos::new(2, 0, 0),
                BlockPos::new(3, 0, 0),
                BlockPos::new(4, 0, 0),
            ])
        );
    }

    #[test]
    fn diagonal_movement_follows_requested_axes_after_collision_clipping() {
        let bounds = BoundingBox::new(
            Vector3::new(0.25, 0.25, 0.25),
            Vector3::new(0.75, 0.75, 0.75),
        );
        let mut visited = HashSet::new();
        for_each_block_along_movement(
            bounds,
            Vector3::new(2.0, 0.0, 4.0),
            Some(Vector3::new(8.0, 0.0, 4.0)),
            |pos| {
                visited.insert(pos);
            },
        );
        assert_eq!(
            visited,
            HashSet::from([
                BlockPos::new(0, 0, 0),
                BlockPos::new(1, 0, 0),
                BlockPos::new(2, 0, 0),
                BlockPos::new(2, 0, 1),
                BlockPos::new(2, 0, 2),
                BlockPos::new(2, 0, 3),
                BlockPos::new(2, 0, 4),
            ])
        );
    }

    #[test]
    fn long_movement_limits_sweep_and_still_visits_destination() {
        let bounds = BoundingBox::new(
            Vector3::new(0.25, 0.25, 0.25),
            Vector3::new(0.75, 0.75, 0.75),
        );
        let movement = Vector3::new(1000.0, 0.0, 0.0);
        let mut visited = HashSet::new();
        for_each_block_along_movement(bounds, movement, Some(movement), |pos| {
            visited.insert(pos);
        });
        assert_eq!(visited.len(), 17);
        assert!(visited.contains(&BlockPos::new(15, 0, 0)));
        assert!(!visited.contains(&BlockPos::new(16, 0, 0)));
        assert!(visited.contains(&BlockPos::new(1000, 0, 0)));
    }

    #[test]
    fn merged_movement_matches_vanilla_iteration_cutoff() {
        let bounds = BoundingBox::new(
            Vector3::new(-0.6875, 0.25, -0.6875),
            Vector3::new(0.6875, 0.8125, 0.6875),
        );
        let mut visited = HashSet::new();
        for_each_block_along_movement(bounds, Vector3::new(1000.0, 0.0, 500.0), None, |pos| {
            visited.insert(pos);
        });
        // Recorded from 26.3: the last new cell is at step 14, so no final-box fallback runs.
        assert_eq!(visited.len(), 39);
        assert!(visited.contains(&BlockPos::new(10, 0, 5)));
        assert!(!visited.contains(&BlockPos::new(11, 0, 5)));
        assert!(!visited.contains(&BlockPos::new(1000, 0, 500)));
    }
}
