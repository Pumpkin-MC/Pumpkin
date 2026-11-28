//! Vanilla `NewMinecartBehavior`: the minecart movement of the `minecart_improvements` feature.

use std::sync::Mutex;
use std::sync::atomic::Ordering;

use pumpkin_data::Block;
use pumpkin_data::block_properties::{PoweredRailLikeProperties, RailShape};
use pumpkin_protocol::java::client::play::MinecartStep;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::rot_lerp;
use pumpkin_util::math::vector3::Vector3;

use super::behavior::{
    EPSILON, RIDEABLE_THRESHOLD, Rail, can_be_picked_up, exits, normalize, pushable_by,
};
use super::{AbstractMinecart, MinecartType, is_minecart};
use crate::entity::EntityBase;
use crate::world::World;

/// Vanilla `ON_RAIL_Y_OFFSET`: a cart rides this far above the rail block floor.
pub(super) const ON_RAIL_Y_OFFSET: f64 = 0.1;
/// Vanilla `OPPOSING_SLOPES_REST_AT_SPEED_THRESHOLD`.
const OPPOSING_SLOPES_REST_AT_SPEED_THRESHOLD: f64 = 0.005;
pub(super) const SLIDE_SPEED: f64 = 0.007_812_5;
/// Share of its speed a cart gains on a slope, when that beats [`SLIDE_SPEED`].
pub(super) const SLOPE_SPEED_SHARE: f64 = 0.02;
const POWERED_RAIL_BOOST: f64 = 0.06;
pub(super) const POWERED_RAIL_START: f64 = 0.2;
const PICKUP_INFLATE: f64 = 0.2;
const PUSH_INFLATE: f64 = 1.0e-7;

/// Vanilla `NewMinecartBehavior` state.
pub struct NewMinecartBehavior {
    /// Vanilla `lerpSteps`, sent and cleared by the entity tracker.
    pub lerp_steps: Mutex<Vec<MinecartStep>>,
}

/// Vanilla `NewMinecartBehavior.TrackIteration`.
struct TrackIteration {
    movement_left: f64,
    first_iteration: bool,
    has_gained_slope_speed: bool,
    has_halted: bool,
    has_boosted: bool,
}

impl TrackIteration {
    const fn should_iterate(&self) -> bool {
        self.first_iteration || self.movement_left > EPSILON
    }
}

/// Vanilla `adjustToRails` `instant`: an instant snap weighs nothing in the client lerp.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Snap {
    Instant,
    Lerped,
}

const fn horizontal(vector: Vector3<f64>) -> Vector3<f64> {
    Vector3::new(vector.x, 0.0, vector.z)
}

fn half(exit: Vector3<i32>) -> Vector3<f64> {
    Vector3::new(
        f64::from(exit.x) * 0.5,
        f64::from(exit.y) * 0.5,
        f64::from(exit.z) * 0.5,
    )
}

/// Vanilla `Vec3.atBottomCenterOf`.
fn bottom_center(pos: BlockPos) -> Vector3<f64> {
    Vector3::new(
        f64::from(pos.0.x) + 0.5,
        f64::from(pos.0.y),
        f64::from(pos.0.z) + 0.5,
    )
}

/// `180 - atan2(z, x)` in degrees: the yaw facing along `x`/`z`.
fn yaw_along(x: f64, z: f64) -> f32 {
    180.0 - (z.atan2(x) * 180.0 / std::f64::consts::PI) as f32
}

/// Whether `movement` heads down the slope of `shape`.
const fn is_descending(movement: Vector3<f64>, shape: RailShape) -> bool {
    match shape {
        RailShape::AscendingEast => movement.x < 0.0,
        RailShape::AscendingWest => movement.x > 0.0,
        RailShape::AscendingNorth => movement.z > 0.0,
        RailShape::AscendingSouth => movement.z < 0.0,
        _ => false,
    }
}

impl NewMinecartBehavior {
    pub(super) const fn new() -> Self {
        Self {
            lerp_steps: Mutex::new(Vec::new()),
        }
    }

    fn push_step(
        &self,
        position: Vector3<f64>,
        movement: Vector3<f64>,
        rotation: (f32, f32),
        weight: f32,
    ) {
        self.lerp_steps
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(MinecartStep {
                position,
                movement,
                yaw: rotation.0,
                pitch: rotation.1,
                weight,
            });
    }
}

impl AbstractMinecart {
    /// Vanilla `NewMinecartBehavior.tick` (server side).
    pub(super) fn tick_new(&self, caller: &dyn EntityBase, behavior: &NewMinecartBehavior) {
        let entity = &self.vehicle.entity;
        let world = entity.world.load();
        if entity.first_tick.load(Ordering::Relaxed) {
            let (pos, rail) = self.current_rail(&world);
            self.on_rails.store(rail.is_some(), Ordering::Relaxed);
            self.adjust_to_rails(behavior, pos, rail, Snap::Instant);
        }
        self.apply_gravity();
        self.move_along_track_new(caller, &world, behavior);
    }

    /// Vanilla `NewMinecartBehavior.adjustToRails`.
    pub(super) fn adjust_to_rails(
        &self,
        behavior: &NewMinecartBehavior,
        pos: BlockPos,
        rail: Option<Rail>,
        snap: Snap,
    ) {
        let Some(rail) = rail else {
            return;
        };
        let entity = &self.vehicle.entity;
        let (exit0, exit1) = exits(rail.shape);
        let (exit0, exit1) = (half(exit0), half(exit1));
        let mut out_direction = horizontal(exit0);
        let mut in_direction = horizontal(exit1);
        let velocity = entity.velocity.load();
        if (velocity.length() > EPSILON
            && velocity.dot(&out_direction) < velocity.dot(&in_direction))
            || is_descending(in_direction, rail.shape)
        {
            std::mem::swap(&mut out_direction, &mut in_direction);
        }

        let flipped = self.flipped.load(Ordering::Relaxed);
        let flip = if flipped { 180.0 } else { 0.0 };
        let mut yaw = yaw_along(out_direction.x, out_direction.z) + flip;
        let previous = entity.pos.load();
        let in_corner = exit0.x != exit1.x && exit0.z != exit1.z;
        let target = if in_corner {
            let from0_to1 = exit1.sub(&exit0);
            let from0_to_pos = previous.sub(&bottom_center(pos)).sub(&exit0);
            let travel = from0_to1 * (from0_to1.dot(&from0_to_pos) / from0_to1.dot(&from0_to1));
            yaw = yaw_along(travel.x, travel.z) + flip;
            bottom_center(pos).add(&exit0).add(&travel)
        } else {
            let z_snap = exit0.x != exit1.x;
            let x_snap = exit0.z != exit1.z;
            Vector3::new(
                if x_snap {
                    f64::from(pos.0.x) + 0.5
                } else {
                    previous.x
                },
                f64::from(pos.0.y),
                if z_snap {
                    f64::from(pos.0.z) + 0.5
                } else {
                    previous.z
                },
            )
        };
        entity.set_pos(previous.add(&target.sub(&previous)));

        let mut pitch = 0.0;
        if exit0.y == exit1.y {
            entity.set_pos(entity.pos.load().add_raw(0.0, ON_RAIL_Y_OFFSET, 0.0));
        } else {
            let in_position = bottom_center(pos).add(&in_direction);
            let distance = in_position.sub(&entity.pos.load()).length();
            entity.set_pos(
                entity
                    .pos
                    .load()
                    .add_raw(0.0, distance + ON_RAIL_Y_OFFSET, 0.0),
            );
            pitch = if flipped { 45.0 } else { -45.0 };
        }

        self.set_rotation(yaw, pitch);
        let position = entity.pos.load();
        let adjust_distance = previous.sub(&position).length();
        if adjust_distance > 0.0 {
            let weight = match snap {
                Snap::Instant => 0.0,
                Snap::Lerped => adjust_distance as f32,
            };
            behavior.push_step(position, entity.velocity.load(), self.rotation(), weight);
        }
    }

    fn rotation(&self) -> (f32, f32) {
        let entity = &self.vehicle.entity;
        (entity.yaw.load(), entity.pitch.load())
    }

    /// Vanilla `NewMinecartBehavior.setRotation`: a near half turn flips the cart instead.
    fn set_rotation(&self, mut yaw: f32, mut pitch: f32) {
        let entity = &self.vehicle.entity;
        let turn = (yaw - entity.yaw.load()).abs();
        if (175.0..=185.0).contains(&turn) {
            self.flipped.fetch_xor(true, Ordering::Relaxed);
            yaw -= 180.0;
            pitch *= -1.0;
        }
        entity.set_pitch(pitch.clamp(-45.0, 45.0) % 360.0);
        entity.yaw.store(yaw % 360.0);
    }

    /// Vanilla `NewMinecartBehavior.moveAlongTrack`.
    fn move_along_track_new(
        &self,
        caller: &dyn EntityBase,
        world: &World,
        behavior: &NewMinecartBehavior,
    ) {
        let entity = &self.vehicle.entity;
        let mut iteration = TrackIteration {
            movement_left: 0.0,
            first_iteration: true,
            has_gained_slope_speed: false,
            has_halted: false,
            has_boosted: false,
        };
        while iteration.should_iterate() && entity.is_alive() {
            let initial = entity.velocity.load();
            let (pos, rail) = self.current_rail(world);
            if self.on_rails.load(Ordering::Relaxed) != rail.is_some() {
                self.on_rails.store(rail.is_some(), Ordering::Relaxed);
                self.adjust_to_rails(behavior, pos, rail, Snap::Lerped);
            }

            if let Some(rail) = rail {
                entity.fall_distance.store(0.0);
                entity.last_pos.store(entity.pos.load());
                if rail.block == &Block::ACTIVATOR_RAIL {
                    self.activate(PoweredRailLikeProperties::from_state_id(rail.state).powered);
                }
                let velocity = self.track_speed(world, horizontal(initial), &mut iteration, rail);
                if iteration.first_iteration {
                    iteration.movement_left = velocity.horizontal_length();
                } else {
                    iteration.movement_left +=
                        velocity.horizontal_length() - initial.horizontal_length();
                }
                entity.velocity.store(velocity);
                iteration.movement_left =
                    self.step_along_track(caller, world, rail, iteration.movement_left);
            } else {
                self.come_off_track(caller);
                iteration.movement_left = 0.0;
            }

            let step_position = entity.pos.load();
            let step_delta = step_position.sub(&entity.last_pos.load());
            let step_length = step_delta.length();
            if step_length > EPSILON {
                let on_rails = self.on_rails.load(Ordering::Relaxed);
                let on_ground = entity.on_ground.load(Ordering::Relaxed);
                if step_delta.horizontal_length_squared() > EPSILON {
                    let flipped = self.flipped.load(Ordering::Relaxed);
                    let mut yaw = yaw_along(step_delta.x, step_delta.z);
                    let mut pitch = if on_ground && !on_rails {
                        0.0
                    } else {
                        90.0 - (step_delta.horizontal_length().atan2(step_delta.y) * 180.0
                            / std::f64::consts::PI) as f32
                    };
                    if flipped {
                        yaw += 180.0;
                        pitch *= -1.0;
                    }
                    self.set_rotation(yaw, pitch);
                } else if !on_rails {
                    let pitch = entity.pitch.load();
                    entity.set_pitch(if on_ground {
                        0.0
                    } else {
                        rot_lerp(0.2, pitch, 0.0)
                    });
                }
                let weight = step_length.min(self.behavior_max_speed()) as f32;
                behavior.push_step(
                    step_position,
                    entity.velocity.load(),
                    self.rotation(),
                    weight,
                );
            } else if initial.horizontal_length_squared() > 0.0 {
                behavior.push_step(step_position, entity.velocity.load(), self.rotation(), 1.0);
            }

            if step_length > EPSILON || iteration.first_iteration {
                // Vanilla applies the block effects twice per step.
                // TODO: vanilla `applyEffectsFromBlocks` also runs `stepOn` and the fire checks,
                // `tick_block_collisions` only the inside-block effects.
                entity.tick_block_collisions(caller);
                entity.tick_block_collisions(caller);
            }
            iteration.first_iteration = false;
        }
    }

    /// Vanilla `calculateTrackSpeed`: each speed change happens once per tick.
    fn track_speed(
        &self,
        world: &World,
        velocity: Vector3<f64>,
        iteration: &mut TrackIteration,
        rail: Rail,
    ) -> Vector3<f64> {
        let mut velocity = velocity;
        if !iteration.has_gained_slope_speed {
            let sloped = self.slope_speed(velocity, rail.shape);
            if sloped.horizontal_length_squared() != velocity.horizontal_length_squared() {
                iteration.has_gained_slope_speed = true;
                velocity = sloped;
            }
        }
        if iteration.first_iteration {
            let input = self.player_input_speed(velocity);
            if input.horizontal_length_squared() != velocity.horizontal_length_squared() {
                iteration.has_halted = true;
                velocity = input;
            }
        }
        if !iteration.has_halted {
            let halted = Self::halt_speed(velocity, rail);
            if halted.horizontal_length_squared() != velocity.horizontal_length_squared() {
                iteration.has_halted = true;
                velocity = halted;
            }
        }
        if iteration.first_iteration {
            velocity = self.apply_natural_slowdown(velocity);
            if velocity.length_squared() > 0.0 {
                let speed = velocity.length().min(self.max_speed());
                velocity = normalize(velocity) * speed;
            }
        }
        if !iteration.has_boosted {
            let boosted = Self::boost_speed(world, velocity, rail);
            if boosted.horizontal_length_squared() != velocity.horizontal_length_squared() {
                iteration.has_boosted = true;
                velocity = boosted;
            }
        }
        velocity
    }

    /// Vanilla `calculateSlopeSpeed`.
    fn slope_speed(&self, velocity: Vector3<f64>, shape: RailShape) -> Vector3<f64> {
        let mut slide = SLIDE_SPEED.max(velocity.horizontal_length() * SLOPE_SPEED_SHARE);
        if self.vehicle.entity.is_in_water() {
            slide *= 0.2;
        }
        match shape {
            RailShape::AscendingEast => velocity.add_raw(-slide, 0.0, 0.0),
            RailShape::AscendingWest => velocity.add_raw(slide, 0.0, 0.0),
            RailShape::AscendingNorth => velocity.add_raw(0.0, 0.0, slide),
            RailShape::AscendingSouth => velocity.add_raw(0.0, 0.0, -slide),
            _ => velocity,
        }
    }

    /// Vanilla `calculatePlayerInputSpeed`: a rider nudges a nearly still cart.
    fn player_input_speed(&self, velocity: Vector3<f64>) -> Vector3<f64> {
        match self.rider_intent() {
            Some(intent) if velocity.horizontal_length_squared() < RIDEABLE_THRESHOLD => {
                let rider = normalize(intent);
                velocity.add(&(normalize(horizontal(rider)) * 0.001))
            }
            _ => velocity,
        }
    }

    /// Vanilla `calculateHaltTrackSpeed`.
    fn halt_speed(velocity: Vector3<f64>, rail: Rail) -> Vector3<f64> {
        match rail.powered_rail() {
            Some(false) if velocity.length() < 0.03 => Vector3::new(0.0, 0.0, 0.0),
            Some(false) => velocity * 0.5,
            _ => velocity,
        }
    }

    /// Vanilla `calculateBoostTrackSpeed`.
    fn boost_speed(world: &World, velocity: Vector3<f64>, rail: Rail) -> Vector3<f64> {
        if rail.powered_rail() != Some(true) {
            return velocity;
        }
        let speed = velocity.length();
        if speed > 0.01 {
            return normalize(velocity) * (speed + POWERED_RAIL_BOOST);
        }
        let direction = Self::redstone_direction(world, rail);
        if direction.length_squared() <= 0.0 {
            velocity
        } else {
            direction * (speed + POWERED_RAIL_START)
        }
    }

    /// Vanilla `NewMinecartBehavior.stepAlongTrack`: moves up to the rail's exit, returns
    /// the movement left for the next rail.
    fn step_along_track(
        &self,
        caller: &dyn EntityBase,
        world: &World,
        rail: Rail,
        movement_left: f64,
    ) -> f64 {
        let entity = &self.vehicle.entity;
        if movement_left < EPSILON {
            return 0.0;
        }
        let old_position = entity.pos.load();
        let (exit0, exit1) = exits(rail.shape);
        let mut movement = horizontal(entity.velocity.load());
        if movement.length() < EPSILON {
            entity.velocity.store(Vector3::new(0.0, 0.0, 0.0));
            return 0.0;
        }

        let in_hill = exit0.y != exit1.y;
        let in_direction = horizontal(half(exit1));
        let mut out_direction = horizontal(half(exit0));
        if movement.dot(&out_direction) < movement.dot(&in_direction) {
            out_direction = in_direction;
        }
        let mut out_position = bottom_center(rail.pos)
            .add(&out_direction)
            .add_raw(0.0, ON_RAIL_Y_OFFSET, 0.0)
            .add(&(normalize(out_direction) * EPSILON));
        if in_hill && !is_descending(movement, rail.shape) {
            out_position = out_position.add_raw(0.0, 1.0, 0.0);
        }

        let towards_out = normalize(out_position.sub(&old_position));
        movement = towards_out * (movement.length() / towards_out.horizontal_length());
        let step = if in_hill {
            f64::from(std::f32::consts::SQRT_2)
        } else {
            1.0
        };
        let mut new_position = old_position.add(&(normalize(movement) * (movement_left * step)));
        let movement_left = if old_position.squared_distance_to_vec(&out_position)
            <= old_position.squared_distance_to_vec(&new_position)
        {
            let left = horizontal(out_position.sub(&new_position)).length();
            new_position = out_position;
            left
        } else {
            0.0
        };

        self.move_entity(caller, new_position.sub(&old_position));
        if in_hill {
            let (block, state) = world.get_block_and_state_id(&BlockPos::floored_v(new_position));
            if let Some(new_shape) = crate::block::blocks::redstone::rails::rail_shape(block, state)
                && self.rest_at_v_shape(rail.shape, new_shape)
            {
                return 0.0;
            }
            let position = entity.pos.load();
            let distance = horizontal(out_position).sub(&horizontal(position)).length();
            let projected_y = out_position.y
                + if is_descending(movement, rail.shape) {
                    distance
                } else {
                    -distance
                };
            if position.y < projected_y {
                entity.set_pos(Vector3::new(position.x, projected_y, position.z));
            }
        }

        if entity.pos.load().sub(&old_position).length() < EPSILON
            && new_position.sub(&old_position).length() > EPSILON
        {
            entity.velocity.store(Vector3::new(0.0, 0.0, 0.0));
            return 0.0;
        }
        entity.velocity.store(movement);
        movement_left
    }

    /// Vanilla `restAtVShape`: a slow cart stops in the dip between two slopes.
    fn rest_at_v_shape(&self, current: RailShape, new: RailShape) -> bool {
        let entity = &self.vehicle.entity;
        let velocity = entity.velocity.load();
        let rests = velocity.length_squared() < OPPOSING_SLOPES_REST_AT_SPEED_THRESHOLD
            && new.is_ascending()
            && is_descending(velocity, current)
            && !is_descending(velocity, new);
        if rests {
            entity.velocity.store(Vector3::new(0.0, 0.0, 0.0));
        }
        rests
    }

    /// Vanilla `AbstractMinecart.move` (experimental): pushes and pickups happen in the move,
    /// a cart that only picked something up keeps going.
    pub(super) fn move_new(&self, caller: &dyn EntityBase, motion: Vector3<f64>) {
        let entity = &self.vehicle.entity;
        let target = entity.pos.load().add(&motion);
        entity.move_entity(caller, motion);
        if self.push_and_pickup_entities_new(caller) {
            entity.move_entity(caller, target.sub(&entity.pos.load()));
        }
    }

    /// Vanilla `NewMinecartBehavior.pushAndPickupEntities`: `true` to finish the move.
    fn push_and_pickup_entities_new(&self, caller: &dyn EntityBase) -> bool {
        let entity = &self.vehicle.entity;
        let bounding_box = entity.bounding_box.load();
        let picked_up = self.pickup_entities(
            caller,
            bounding_box.expand(PICKUP_INFLATE, 0.0, PICKUP_INFLATE),
        );
        if !entity.horizontal_collision.load(Ordering::Relaxed)
            && !entity.vertical_collision.load(Ordering::Relaxed)
        {
            return false;
        }
        let pushed = self.push_entities(
            caller,
            bounding_box.expand(PUSH_INFLATE, PUSH_INFLATE, PUSH_INFLATE),
        );
        picked_up && !pushed
    }

    /// Vanilla `NewMinecartBehavior.pickupEntities`.
    fn pickup_entities(
        &self,
        caller: &dyn EntityBase,
        hitbox: pumpkin_util::math::boundingbox::BoundingBox,
    ) -> bool {
        let entity = &self.vehicle.entity;
        if self.minecart_type() != MinecartType::Rideable || entity.has_passengers() {
            return false;
        }
        let world = entity.world.load();
        let own_team = caller.get_team();
        for other in world.get_all_at_box(&hitbox) {
            if pushable_by(entity.entity_id, own_team.as_ref(), other.as_ref())
                && !entity.has_passengers()
                && can_be_picked_up(other.as_ref())
                && Self::start_riding(entity, &world, other)
            {
                return true;
            }
        }
        false
    }

    /// Vanilla `NewMinecartBehavior.pushEntities`.
    fn push_entities(
        &self,
        caller: &dyn EntityBase,
        hitbox: pumpkin_util::math::boundingbox::BoundingBox,
    ) -> bool {
        let entity = &self.vehicle.entity;
        let world = entity.world.load();
        let mut pushed = false;
        if self.minecart_type() == MinecartType::Rideable {
            let own_team = caller.get_team();
            for other in world.get_all_at_box(&hitbox) {
                if pushable_by(entity.entity_id, own_team.as_ref(), other.as_ref())
                    && (entity.has_passengers() || !can_be_picked_up(other.as_ref()))
                {
                    other.push(caller);
                    pushed = true;
                }
            }
        } else {
            for other in world.get_entities_of_type_at_box(&hitbox, is_minecart) {
                if other.get_entity().entity_id != entity.entity_id
                    && !entity.has_passenger(other.get_entity().entity_id)
                    && other.is_pushable()
                {
                    other.push(caller);
                    pushed = true;
                }
            }
        }
        pushed
    }

    /// Vanilla `MinecartCollisionContext`: the block under the rail and the block a slope
    /// leans on don't collide.
    pub(super) fn ignored_collisions(&self) -> Option<(BlockPos, Option<BlockPos>)> {
        let world = self.vehicle.entity.world.load();
        let (pos, rail) = self.current_rail(&world);
        let rail = rail?;
        let slope = match rail.shape {
            RailShape::AscendingEast => Some(pos.east()),
            RailShape::AscendingWest => Some(pos.west()),
            RailShape::AscendingNorth => Some(pos.north()),
            RailShape::AscendingSouth => Some(pos.south()),
            _ => None,
        };
        Some((pos.down(), slope))
    }
}
