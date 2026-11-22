use std::collections::HashSet;
use std::sync::atomic::Ordering;

use crate::entity::ai::control::move_control::Operation;
use crate::entity::ai::control::{Control, MoveControlTrait};
use crate::entity::mob::Mob;
use pumpkin_data::Block;
use pumpkin_data::tag::Taggable;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

pub struct GhastMoveControl {
    pub wanted: Vector3<f64>,
    pub speed_modifier: f64,
    pub operation: Operation,
    pub float_duration: i32,
    pub careful: bool,
    pub is_still: Option<fn(&dyn Mob) -> bool>,
}

impl GhastMoveControl {
    #[must_use]
    pub fn new(careful: bool, is_still: Option<fn(&dyn Mob) -> bool>) -> Self {
        Self {
            wanted: Vector3::default(),
            speed_modifier: 1.0,
            operation: Operation::Wait,
            float_duration: 0,
            careful,
            is_still,
        }
    }

    fn can_reach(&self, mob: &dyn Mob, travel: Vector3<f64>) -> bool {
        let aabb = mob.get_entity().bounding_box.load();
        let aabb_at_destination = aabb.shift(travel);
        if self.careful {
            let inflated = aabb_at_destination.expand_all(1.0);
            let start = BlockPos::floored_v(inflated.min);
            let end = BlockPos::floored_v(inflated.max);
            for pos in BlockPos::iterate(start, end) {
                if !Self::block_traversal_possible(mob, None, None, pos, false, false, true) {
                    return false;
                }
            }
        }
        let is_in_water = mob.get_entity().is_in_water();
        let is_in_lava = mob.get_entity().touching_lava.load(Ordering::Relaxed);
        let start = mob.get_entity().pos.load();
        let end = start + travel;

        for_each_block_intersected_between(start, end, &aabb_at_destination, |block_pos| {
            if aabb.intersects(&BoundingBox::from_block(&block_pos)) {
                true
            } else {
                Self::block_traversal_possible(
                    mob,
                    Some(start),
                    Some(end),
                    block_pos,
                    is_in_water,
                    is_in_lava,
                    self.careful,
                )
            }
        })
    }

    fn block_traversal_possible(
        mob: &dyn Mob,
        start: Option<Vector3<f64>>,
        end: Option<Vector3<f64>>,
        pos: BlockPos,
        can_path_through_water: bool,
        can_path_through_lava: bool,
        careful: bool,
    ) -> bool {
        let world = mob.get_entity().world.load();
        let state = world.get_block_state(&pos);
        if state.is_air() {
            return true;
        }

        let path_no_collisions = if let (Some(start), Some(end)) = (start, end) {
            let travel = end - start;
            let mob_bb = mob.get_entity().bounding_box.load();
            let mut collided = false;
            for shape in state.get_block_collision_shapes_at(&pos) {
                if box_collided_along_vector(&mob_bb, travel, &shape) {
                    collided = true;
                    break;
                }
            }
            !collided
        } else {
            state.collision_shapes.is_empty()
        };

        if !careful {
            return path_no_collisions;
        }

        let block = Block::from_state_id(state.id);
        if block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_HAPPY_GHAST_AVOIDS) {
            return false;
        }

        let (_, fluid_state) = world.get_fluid_and_fluid_state(&pos);
        if !fluid_state.is_empty {
            let fluid_collided = if let (Some(start), Some(end)) = (start, end) {
                let travel = end - start;
                let mob_bb = mob.get_entity().bounding_box.load();
                let fluid_box = BoundingBox::from_block(&pos);
                box_collided_along_vector(&mob_bb, travel, &fluid_box)
            } else {
                true
            };

            if fluid_collided {
                if world.fluid_state_has_tag(&pos, &pumpkin_data::tag::Fluid::MINECRAFT_WATER) {
                    if !can_path_through_water {
                        return false;
                    }
                } else if world.fluid_state_has_tag(&pos, &pumpkin_data::tag::Fluid::MINECRAFT_LAVA)
                    && !can_path_through_lava
                {
                    return false;
                }
            }
        }

        path_no_collisions
    }
}

fn box_collided_along_vector(bb: &BoundingBox, vector: Vector3<f64>, shape: &BoundingBox) -> bool {
    let from = bb.get_center();
    let to = from + vector;
    let size_x = bb.max.x - bb.min.x;
    let size_y = bb.max.y - bb.min.y;
    let size_z = bb.max.z - bb.min.z;
    let inflated = shape.expand(
        size_x * 0.5 - 1.0e-7,
        size_y * 0.5 - 1.0e-7,
        size_z * 0.5 - 1.0e-7,
    );
    if inflated.contains(to) || inflated.contains(from) {
        return true;
    }
    inflated.clip(from, to).is_some()
}

fn mth_sign(x: f64) -> i32 {
    match x.partial_cmp(&0.0) {
        Some(std::cmp::Ordering::Greater) => 1,
        Some(std::cmp::Ordering::Less) => -1,
        _ => 0,
    }
}

fn compute_t(sign: i32, t_delta: f64, from_corner: f64) -> f64 {
    let frac = from_corner - from_corner.floor();
    match sign.cmp(&0) {
        std::cmp::Ordering::Greater => t_delta * (1.0 - frac),
        std::cmp::Ordering::Less => t_delta * frac,
        std::cmp::Ordering::Equal => f64::MAX,
    }
}

fn for_each_block_intersected_between<F>(
    from: Vector3<f64>,
    to: Vector3<f64>,
    aabb_at_target: &BoundingBox,
    mut visitor: F,
) -> bool
where
    F: FnMut(BlockPos) -> bool,
{
    let travel = to - from;
    if travel.length_squared() < 1.0e-5 * 1.0e-5 {
        let min_pos = BlockPos::floored_v(aabb_at_target.min);
        let max_pos = BlockPos::floored_v(aabb_at_target.max);
        for pos in BlockPos::iterate(min_pos, max_pos) {
            if !visitor(pos) {
                return false;
            }
        }
        return true;
    }

    let mut visited_blocks = HashSet::new();

    let aabb_at_start = aabb_at_target.shift(travel * -1.0);
    let start_min = BlockPos::floored_v(aabb_at_start.min);
    let start_max = BlockPos::floored_v(aabb_at_start.max);
    for pos in BlockPos::iterate(start_min, start_max) {
        if !visitor(pos) {
            return false;
        }
        visited_blocks.insert(pos);
    }

    if add_collisions_along_travel(&mut visited_blocks, travel, aabb_at_target, &mut visitor) < 0 {
        return false;
    }

    let target_min = BlockPos::floored_v(aabb_at_target.min);
    let target_max = BlockPos::floored_v(aabb_at_target.max);
    for pos in BlockPos::iterate(target_min, target_max) {
        if visited_blocks.insert(pos) && !visitor(pos) {
            return false;
        }
    }

    true
}

fn add_collisions_along_travel<F>(
    visited_blocks: &mut HashSet<BlockPos>,
    delta_move: Vector3<f64>,
    aabb_at_target: &BoundingBox,
    visitor: &mut F,
) -> i32
where
    F: FnMut(BlockPos) -> bool,
{
    let box_size_x = aabb_at_target.max.x - aabb_at_target.min.x;
    let box_size_y = aabb_at_target.max.y - aabb_at_target.min.y;
    let box_size_z = aabb_at_target.max.z - aabb_at_target.min.z;

    let corner_dir = get_furthest_corner(delta_move);
    let to_center = aabb_at_target.get_center();
    let to_corner = Vector3::new(
        to_center.x + box_size_x * 0.5 * f64::from(corner_dir.x),
        to_center.y + box_size_y * 0.5 * f64::from(corner_dir.y),
        to_center.z + box_size_z * 0.5 * f64::from(corner_dir.z),
    );
    let from_corner = to_corner - delta_move;

    let mut corner_visited_block_x = from_corner.x.floor() as i32;
    let mut corner_visited_block_y = from_corner.y.floor() as i32;
    let mut corner_visited_block_z = from_corner.z.floor() as i32;

    let sign_x = mth_sign(delta_move.x);
    let sign_y = mth_sign(delta_move.y);
    let sign_z = mth_sign(delta_move.z);

    let t_delta_x = if sign_x == 0 {
        f64::MAX
    } else {
        f64::from(sign_x) / delta_move.x
    };
    let t_delta_y = if sign_y == 0 {
        f64::MAX
    } else {
        f64::from(sign_y) / delta_move.y
    };
    let t_delta_z = if sign_z == 0 {
        f64::MAX
    } else {
        f64::from(sign_z) / delta_move.z
    };

    let mut t_x = compute_t(sign_x, t_delta_x, from_corner.x);
    let mut t_y = compute_t(sign_y, t_delta_y, from_corner.y);
    let mut t_z = compute_t(sign_z, t_delta_z, from_corner.z);

    let mut iterations = 0;

    while t_x <= 1.0 || t_y <= 1.0 || t_z <= 1.0 {
        if t_x < t_y {
            if t_x < t_z {
                corner_visited_block_x += sign_x;
                t_x += t_delta_x;
            } else {
                corner_visited_block_z += sign_z;
                t_z += t_delta_z;
            }
        } else if t_y < t_z {
            corner_visited_block_y += sign_y;
            t_y += t_delta_y;
        } else {
            corner_visited_block_z += sign_z;
            t_z += t_delta_z;
        }

        let clip_box = BoundingBox::new(
            Vector3::new(
                f64::from(corner_visited_block_x),
                f64::from(corner_visited_block_y),
                f64::from(corner_visited_block_z),
            ),
            Vector3::new(
                f64::from(corner_visited_block_x + 1),
                f64::from(corner_visited_block_y + 1),
                f64::from(corner_visited_block_z + 1),
            ),
        );
        let Some(hit_point) = clip_box.clip(from_corner, to_corner) else {
            continue;
        };

        iterations += 1;

        let corner_hit_x = hit_point.x.clamp(
            f64::from(corner_visited_block_x) + 1.0e-5,
            f64::from(corner_visited_block_x + 1) - 1.0e-5,
        );
        let corner_hit_y = hit_point.y.clamp(
            f64::from(corner_visited_block_y) + 1.0e-5,
            f64::from(corner_visited_block_y + 1) - 1.0e-5,
        );
        let corner_hit_z = hit_point.z.clamp(
            f64::from(corner_visited_block_z) + 1.0e-5,
            f64::from(corner_visited_block_z + 1) - 1.0e-5,
        );

        let opposite_corner_x =
            (corner_hit_x - box_size_x * f64::from(corner_dir.x)).floor() as i32;
        let opposite_corner_y =
            (corner_hit_y - box_size_y * f64::from(corner_dir.y)).floor() as i32;
        let opposite_corner_z =
            (corner_hit_z - box_size_z * f64::from(corner_dir.z)).floor() as i32;

        let pos1 = BlockPos::new(
            corner_visited_block_x,
            corner_visited_block_y,
            corner_visited_block_z,
        );
        let pos2 = BlockPos::new(opposite_corner_x, opposite_corner_y, opposite_corner_z);
        for pos in BlockPos::iterate(pos1, pos2) {
            if visited_blocks.insert(pos) && !visitor(pos) {
                return -1;
            }
        }
    }

    iterations
}

fn get_furthest_corner(direction: Vector3<f64>) -> Vector3<i32> {
    let x_dot = direction.x.abs();
    let y_dot = direction.y.abs();
    let z_dot = direction.z.abs();

    let x_sign = if direction.x >= 0.0 { 1 } else { -1 };
    let y_sign = if direction.y >= 0.0 { 1 } else { -1 };
    let z_sign = if direction.z >= 0.0 { 1 } else { -1 };

    if x_dot <= y_dot && x_dot <= z_dot {
        Vector3::new(-x_sign, -z_sign, y_sign)
    } else if y_dot <= z_dot {
        Vector3::new(z_sign, -y_sign, -x_sign)
    } else {
        Vector3::new(-y_sign, x_sign, -z_sign)
    }
}

impl Control for GhastMoveControl {}

impl MoveControlTrait for GhastMoveControl {
    fn tick(&mut self, mob: &dyn Mob) {
        if self.is_still.is_some_and(|check| check(mob)) {
            self.operation = Operation::Wait;
            let entity = mob.get_entity();
            let vel = entity.velocity.load();
            if vel.x != 0.0 || vel.y != 0.0 || vel.z != 0.0 {
                entity.velocity.store(Vector3::default());
                entity.velocity_dirty.store(true, Ordering::SeqCst);
            }
            mob.get_mob_entity().steered().navigator().stop();
            return;
        }

        if self.operation == Operation::MoveTo {
            let timeout = self.float_duration <= 0;
            self.float_duration -= 1;
            if timeout {
                self.float_duration += mob.get_random().random_range(2..=6);
                let pos = mob.get_entity().pos.load();
                let travel = self.wanted - pos;
                if self.can_reach(mob, travel) {
                    let entity = mob.get_entity();
                    let current_vel = entity.velocity.load();
                    let flying_speed =
                        mob.get_mob_entity().living_entity.get_attribute_value(
                            &pumpkin_data::attributes::Attributes::FLYING_SPEED,
                        ) * 5.0
                            / 3.0;
                    entity
                        .velocity
                        .store(current_vel + travel.normalize() * flying_speed);
                } else {
                    self.operation = Operation::Wait;
                }
            }
        }
    }

    fn set_wanted_position(&mut self, x: f64, y: f64, z: f64, speed_modifier: f64) {
        self.wanted = Vector3::new(x, y, z);
        self.speed_modifier = speed_modifier;
        self.operation = Operation::MoveTo;
    }

    fn has_wanted(&self) -> bool {
        self.operation == Operation::MoveTo
    }

    fn get_wanted_position(&self) -> Vector3<f64> {
        self.wanted
    }

    fn set_wait(&mut self) {
        self.operation = Operation::Wait;
    }
}
