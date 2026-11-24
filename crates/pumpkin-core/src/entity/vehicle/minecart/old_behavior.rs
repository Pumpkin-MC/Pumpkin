//! Vanilla `OldMinecartBehavior`: the default minecart movement.

use std::sync::atomic::Ordering;

use pumpkin_data::Block;
use pumpkin_data::block_properties::{PoweredRailLikeProperties, RailShape};
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::{atan2, wrap_degrees};

use super::behavior::{RIDEABLE_THRESHOLD, Rail, can_be_picked_up, exits, pushable_by};
use super::{AbstractMinecart, MinecartType, RAIL_HEIGHT, is_minecart};
use crate::entity::EntityBase;
use crate::world::World;

/// Downhill acceleration per tick on a slope.
const SLIDE_SPEED: f64 = 0.007_812_5;
const POWERED_RAIL_BOOST: f64 = 0.06;
/// Kick off a powered rail that ends at a solid block.
const POWERED_RAIL_KICK: f64 = 0.02;
const PUSH_INFLATE: f64 = 0.2f32 as f64;
/// Vanilla `getMaxSpeed`.
pub(super) const MAX_SPEED: f64 = 0.4;
pub(super) const MAX_SPEED_IN_WATER: f64 = 0.2;

impl Rail {
    /// Vanilla `OldMinecartBehavior.getPos` on this rail.
    fn point(&self, position: Vector3<f64>) -> Vector3<f64> {
        let (exit0, exit1) = exits(self.shape);
        let base = self.pos.0.to_f64();
        let end = |exit: Vector3<i32>| {
            Vector3::new(
                base.x + 0.5 + f64::from(exit.x) * 0.5,
                base.y + RAIL_HEIGHT + f64::from(exit.y) * 0.5,
                base.z + 0.5 + f64::from(exit.z) * 0.5,
            )
        };
        let (start, stop) = (end(exit0), end(exit1));
        let xd = stop.x - start.x;
        let yd = (stop.y - start.y) * 2.0;
        let zd = stop.z - start.z;
        let progress = track_progress(position, base, start, xd, zd);
        let mut point = Vector3::new(
            start.x + xd * progress,
            start.y + yd * progress,
            start.z + zd * progress,
        );
        if yd < 0.0 {
            point.y += 1.0;
        } else if yd > 0.0 {
            point.y += 0.5;
        }
        point
    }
}

/// How far along the rail from `start` the position is.
fn track_progress(
    position: Vector3<f64>,
    base: Vector3<f64>,
    start: Vector3<f64>,
    xd: f64,
    zd: f64,
) -> f64 {
    if xd == 0.0 {
        position.z - base.z
    } else if zd == 0.0 {
        position.x - base.x
    } else {
        ((position.x - start.x) * xd + (position.z - start.z) * zd) * 2.0
    }
}

/// Vanilla `OldMinecartBehavior.getKnownMovement`: clamped to the old speed cap.
pub(super) fn known_movement(movement: Vector3<f64>) -> Vector3<f64> {
    if movement.x.is_nan() || movement.y.is_nan() || movement.z.is_nan() {
        return Vector3::new(0.0, 0.0, 0.0);
    }
    Vector3::new(
        movement.x.clamp(-MAX_SPEED, MAX_SPEED),
        movement.y,
        movement.z.clamp(-MAX_SPEED, MAX_SPEED),
    )
}

impl AbstractMinecart {
    /// Vanilla `OldMinecartBehavior.tick` (server side).
    pub(super) fn tick_old(&self, caller: &dyn EntityBase, last_yaw: f32) {
        let entity = &self.vehicle.entity;
        self.apply_gravity();
        let world = entity.world.load();
        let (_, rail) = self.current_rail(&world);
        self.on_rails.store(rail.is_some(), Ordering::Relaxed);
        if let Some(rail) = rail {
            self.move_along_track_old(caller, &world, rail);
            if rail.block == &Block::ACTIVATOR_RAIL {
                self.activate(PoweredRailLikeProperties::from_state_id(rail.state).powered);
            }
        } else {
            self.come_off_track(caller);
        }
        entity.tick_block_collisions(caller);
        self.update_rotation(last_yaw);
        self.push_and_pickup_entities_old(caller, &world);
    }

    /// Vanilla `OldMinecartBehavior.moveAlongTrack`.
    #[expect(clippy::too_many_lines, reason = "one vanilla method")]
    fn move_along_track_old(&self, caller: &dyn EntityBase, world: &World, rail: Rail) {
        let entity = &self.vehicle.entity;
        entity.fall_distance.store(0.0);
        let position = entity.pos.load();
        // The cart has not moved since `current_rail`, so `getPos` hits the same rail.
        let old_point = rail.point(position);
        let base = rail.pos.0.to_f64();
        let mut y = base.y;

        let powered = rail.powered_rail();
        let power_track = powered == Some(true);
        let mut halt_track = powered == Some(false);

        let in_water = entity.is_in_water();
        let slide = if in_water {
            SLIDE_SPEED * 0.2
        } else {
            SLIDE_SPEED
        };
        let mut velocity = entity.velocity.load();
        match rail.shape {
            RailShape::AscendingEast => velocity.x -= slide,
            RailShape::AscendingWest => velocity.x += slide,
            RailShape::AscendingNorth => velocity.z += slide,
            RailShape::AscendingSouth => velocity.z -= slide,
            _ => {}
        }
        if rail.shape.is_ascending() {
            y += 1.0;
        }

        let (exit0, exit1) = exits(rail.shape);
        let mut xd = f64::from(exit1.x - exit0.x);
        let mut zd = f64::from(exit1.z - exit0.z);
        let length = xd.hypot(zd);
        if velocity.x * xd + velocity.z * zd < 0.0 {
            xd = -xd;
            zd = -zd;
        }
        let speed = velocity.horizontal_length().min(2.0);
        velocity = Vector3::new(speed * xd / length, velocity.y, speed * zd / length);

        if let Some(intent) = self.rider_intent()
            && velocity.horizontal_length_squared() < RIDEABLE_THRESHOLD
        {
            velocity.x += intent.x * 0.001;
            velocity.z += intent.z * 0.001;
            halt_track = false;
        }

        if halt_track {
            velocity = if velocity.horizontal_length() < 0.03 {
                Vector3::new(0.0, 0.0, 0.0)
            } else {
                velocity.multiply(0.5, 0.0, 0.5)
            };
        }

        let x0 = base.x + 0.5 + f64::from(exit0.x) * 0.5;
        let z0 = base.z + 0.5 + f64::from(exit0.z) * 0.5;
        let xd = f64::from(exit1.x - exit0.x) * 0.5;
        let zd = f64::from(exit1.z - exit0.z) * 0.5;
        let progress = track_progress(position, base, Vector3::new(x0, 0.0, z0), xd, zd);
        entity.set_pos(Vector3::new(x0 + xd * progress, y, z0 + zd * progress));
        entity.velocity.store(velocity);

        let scale = if entity.has_passengers() { 0.75 } else { 1.0 };
        let max_speed = self.max_speed();
        self.move_entity(
            caller,
            Vector3::new(
                (scale * velocity.x).clamp(-max_speed, max_speed),
                0.0,
                (scale * velocity.z).clamp(-max_speed, max_speed),
            ),
        );

        let moved = entity.pos.load();
        let column = (moved.x.floor() as i32, moved.z.floor() as i32);
        let reached = |exit: Vector3<i32>| {
            exit.y != 0 && column == (rail.pos.0.x + exit.x, rail.pos.0.z + exit.z)
        };
        if reached(exit0) {
            entity.set_pos(moved.add_raw(0.0, f64::from(exit0.y), 0.0));
        } else if reached(exit1) {
            entity.set_pos(moved.add_raw(0.0, f64::from(exit1.y), 0.0));
        }

        let mut velocity = self.apply_natural_slowdown(entity.velocity.load());
        let moved = entity.pos.load();
        if let (_, Some(new_rail)) = self.current_rail(world) {
            let new_point = new_rail.point(moved);
            let drop = (old_point.y - new_point.y) * 0.05;
            let speed = velocity.horizontal_length();
            if speed > 0.0 {
                let factor = (speed + drop) / speed;
                velocity = velocity.multiply(factor, 1.0, factor);
            }
            entity.set_pos(Vector3::new(moved.x, new_point.y, moved.z));
        }

        let moved = entity.pos.load();
        let (xn, zn) = (moved.x.floor() as i32, moved.z.floor() as i32);
        if xn != rail.pos.0.x || zn != rail.pos.0.z {
            let speed = velocity.horizontal_length();
            velocity = Vector3::new(
                speed * f64::from(xn - rail.pos.0.x),
                velocity.y,
                speed * f64::from(zn - rail.pos.0.z),
            );
        }

        if power_track {
            velocity = Self::powered_rail_push(world, rail, velocity);
        }
        entity.velocity.store(velocity);
    }

    /// `moveAlongTrack` on a powered rail: boost, or a kick away from a solid block.
    fn powered_rail_push(world: &World, rail: Rail, velocity: Vector3<f64>) -> Vector3<f64> {
        let speed = velocity.horizontal_length();
        if speed > 0.01 {
            return velocity.add_raw(
                velocity.x / speed * POWERED_RAIL_BOOST,
                0.0,
                velocity.z / speed * POWERED_RAIL_BOOST,
            );
        }
        let direction = Self::redstone_direction(world, rail);
        let mut velocity = velocity;
        if direction.x != 0.0 {
            velocity.x = direction.x * POWERED_RAIL_KICK;
        } else if direction.z != 0.0 {
            velocity.z = direction.z * POWERED_RAIL_KICK;
        }
        velocity
    }

    /// Vanilla `OldMinecartBehavior.tick` rotation: face the way the cart moved.
    fn update_rotation(&self, last_yaw: f32) {
        let entity = &self.vehicle.entity;
        entity.set_pitch(0.0);
        let mut yaw = entity.yaw.load();
        let last = entity.last_pos.load();
        let pos = entity.pos.load();
        let (x_diff, z_diff) = (last.x - pos.x, last.z - pos.z);
        let flipped = self.flipped.load(Ordering::Relaxed);
        if x_diff * x_diff + z_diff * z_diff > 0.001 {
            yaw = (atan2(z_diff, x_diff) * 180.0 / std::f64::consts::PI) as f32;
            if flipped {
                yaw += 180.0;
            }
        }
        let turn = f64::from(wrap_degrees(yaw - last_yaw));
        if !(-170.0..170.0).contains(&turn) {
            yaw += 180.0;
            self.flipped.store(!flipped, Ordering::Relaxed);
        }
        entity.yaw.store(yaw % 360.0);
    }

    /// Vanilla `OldMinecartBehavior.pushAndPickupEntities`.
    fn push_and_pickup_entities_old(&self, caller: &dyn EntityBase, world: &World) {
        let entity = &self.vehicle.entity;
        let hitbox = entity
            .bounding_box
            .load()
            .expand(PUSH_INFLATE, 0.0, PUSH_INFLATE);
        let picks_up = self.minecart_type() == MinecartType::Rideable
            && entity.velocity.load().horizontal_length_squared() >= RIDEABLE_THRESHOLD;
        if !picks_up {
            for other in world.get_entities_of_type_at_box(&hitbox, is_minecart) {
                if other.get_entity().entity_id != entity.entity_id
                    && !entity.has_passenger(other.get_entity().entity_id)
                    && other.is_pushable()
                {
                    other.push(caller);
                }
            }
            return;
        }

        let own_team = caller.get_team();
        for other in world.get_all_at_box(&hitbox) {
            if !pushable_by(entity.entity_id, own_team.as_ref(), other.as_ref()) {
                continue;
            }
            if !entity.has_passengers() && can_be_picked_up(other.as_ref()) {
                Self::start_riding(entity, world, other);
            } else {
                other.push(caller);
            }
        }
    }
}
