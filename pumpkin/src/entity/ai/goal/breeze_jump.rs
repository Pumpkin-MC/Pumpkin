//! Port of `LongJump.java`.
//!
//! The breeze's signature leap, used both to close distance on its target and to
//! reposition around it (it always aims for a point behind the target - see
//! `breeze_util::random_point_behind_target`).

use std::sync::Weak;
use std::sync::atomic::Ordering::Relaxed;

use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityPose;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::xoroshiro128::Xoroshiro;
use pumpkin_util::random::{RandomGenerator, RandomImpl, get_seed};

use crate::entity::{
    EntityBase,
    ai::goal::{Controls, Goal, GoalFuture, breeze_util::random_point_behind_target},
    mob::{Mob, breeze::BreezeEntity},
};

// LongJump.java constants.
const INHALING_DURATION_TICKS: i32 = 10;
const JUMP_COOLDOWN_TICKS: i32 = 10;
const JUMP_COOLDOWN_WHEN_HURT_TICKS: i32 = 2;
const MAX_JUMP_VELOCITY_MULTIPLIER: f64 = 0.058_333_334;
const REQUIRED_AIR_BLOCKS_ABOVE: i32 = 4;
const ALLOWED_ANGLES: [i32; 5] = [40, 55, 60, 75, 80];
// Sensor.HURT_BY / BreezeAi.TICKS_TO_REMEMBER_SEEN_TARGET: a hit is "recent" for 100 ticks.
const RECENT_HURT_TICKS: i32 = 100;

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Idle,
    Inhaling { ticks_left: i32 },
    Jumping,
}

pub struct BreezeJumpGoal {
    breeze: Weak<BreezeEntity>,
    phase: Phase,
    jump_target: Option<BlockPos>,
    leaving_water: bool,
}

impl BreezeJumpGoal {
    #[must_use]
    pub const fn new(breeze: Weak<BreezeEntity>) -> Self {
        Self {
            breeze,
            phase: Phase::Idle,
            jump_target: None,
            leaving_water: false,
        }
    }

    async fn has_line_of_sight(mob: &dyn Mob, target: Vector3<f64>) -> bool {
        let entity = mob.get_entity();
        entity
            .world
            .load_full()
            .raycast(entity.pos.load(), target, async |block_pos, world| {
                world.get_block_state(block_pos).is_solid()
            })
            .await
            .is_none()
    }

    /// `LongJump.snapToSurface`: clip down 10 blocks for solid ground, falling back to
    /// clipping up 10 blocks if nothing is found below.
    async fn snap_to_surface(mob: &dyn Mob, target: Vector3<f64>) -> Option<BlockPos> {
        let entity = mob.get_entity();
        let world = entity.world.load_full();

        let below = target.sub_raw(0.0, 10.0, 0.0);
        if let Some((hit, _)) = world
            .raycast(target, below, async |block_pos, world| {
                world.get_block_state(block_pos).is_solid()
            })
            .await
        {
            return Some(hit.up());
        }

        let above = target.add_raw(0.0, 10.0, 0.0);
        world
            .raycast(target, above, async |block_pos, world| {
                world.get_block_state(block_pos).is_solid()
            })
            .await
            .map(|(hit, _)| hit.up())
    }

    fn too_close_for_jump(breeze_pos: Vector3<f64>, target_pos: Vector3<f64>) -> bool {
        target_pos.sub(&breeze_pos).length() - 4.0 <= 0.0
    }

    fn out_of_aggro_range(
        breeze_pos: Vector3<f64>,
        target_pos: Vector3<f64>,
        follow_range: f64,
    ) -> bool {
        target_pos.sub(&breeze_pos).length() >= follow_range
    }

    fn can_jump_from_current_position(mob: &dyn Mob) -> bool {
        let entity = mob.get_entity();
        let world = entity.world.load_full();
        let current_pos = entity.block_pos.load();

        let (block, _) = world.get_block_and_state(&current_pos);
        if block.id == pumpkin_data::Block::HONEY_BLOCK.id {
            return false;
        }

        for i in 1..=REQUIRED_AIR_BLOCKS_ABOVE {
            let pos = current_pos.up_height(i);
            let state = world.get_block_state(&pos);
            let fluid = world.get_fluid(&pos);
            let is_water = fluid == &Fluid::WATER || fluid == &Fluid::FLOWING_WATER;
            if !state.is_air() && !is_water {
                return false;
            }
        }

        true
    }

    /// `LongJump.calculateOptimalJumpVector`: tries each allowed launch angle (shuffled)
    /// until one produces a valid ballistic solution under `max_jump_velocity`.
    fn calculate_optimal_jump_vector(
        mob_pos: Vector3<f64>,
        target_pos: Vector3<f64>,
        gravity: f64,
        follow_range: f64,
        random: &mut RandomGenerator,
    ) -> Option<Vector3<f64>> {
        let mut angles = ALLOWED_ANGLES;
        // Fisher-Yates, matching `Util.shuffledCopy`'s effect of trying every angle in a
        // random order.
        for i in (1..angles.len()).rev() {
            let j = (random.next_f32() * (i + 1) as f32) as usize;
            angles.swap(i, j.min(i));
        }

        let max_jump_velocity = MAX_JUMP_VELOCITY_MULTIPLIER * follow_range;
        for angle in angles {
            if let Some(v) = calculate_jump_vector_for_angle(
                mob_pos,
                target_pos,
                gravity,
                max_jump_velocity,
                angle,
            ) {
                return Some(v);
            }
        }
        None
    }
}

/// Port of `LongJumpUtil.calculateJumpVectorForAngle` with `checkCollision = false`.
///
/// This is the only mode the breeze ever uses - a closed-form ballistic-arc solve with
/// no world access, so it is unit-testable in isolation.
#[must_use]
pub fn calculate_jump_vector_for_angle(
    mob_pos: Vector3<f64>,
    target_pos: Vector3<f64>,
    gravity: f64,
    max_jump_velocity: f64,
    angle_deg: i32,
) -> Option<Vector3<f64>> {
    let direction_plane = Vector3::new(target_pos.x - mob_pos.x, 0.0, target_pos.z - mob_pos.z)
        .normalize()
        .multiply(0.5, 0.5, 0.5);
    let aim_point = target_pos.sub(&direction_plane);
    let direction = aim_point.sub(&mob_pos);

    let angle_rad = f64::from(angle_deg) * std::f64::consts::PI / 180.0;
    let xz_angle = direction.z.atan2(direction.x);
    let r2 = direction.horizontal_length_squared();
    let r = r2.sqrt();
    let y = direction.y;

    let sin_2ang = (2.0 * angle_rad).sin();
    let cos_ang_sqr = angle_rad.cos().powi(2);
    let sin_ang = angle_rad.sin();
    let cos_ang = angle_rad.cos();

    let v0_sqr = r2 * gravity / (r * sin_2ang - 2.0 * y * cos_ang_sqr);
    if v0_sqr < 0.0 {
        return None;
    }
    let v0 = v0_sqr.sqrt();
    if v0 > max_jump_velocity {
        return None;
    }

    let v0_r = v0 * cos_ang;
    let v0_y = v0 * sin_ang;
    Some(
        Vector3::new(v0_r * xz_angle.cos(), v0_y, v0_r * xz_angle.sin()).multiply(0.95, 0.95, 0.95),
    )
}

impl Goal for BreezeJumpGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(breeze) = self.breeze.upgrade() else {
                return false;
            };
            // Mutual exclusion with BreezeShootGoal, mirroring BREEZE_SHOOT's presence
            // gating `Shoot` in and `LongJump` out.
            if breeze.shoot_window_ticks() > 0 || breeze.jump_cooldown_ticks() > 0 {
                return false;
            }

            let entity = mob.get_entity();
            let on_ground = entity.on_ground.load(Relaxed);
            let touching_water = entity.touching_water.load(Relaxed);
            if !on_ground && !touching_water {
                return false;
            }

            let Some(target) = breeze.mob_entity.target.lock().await.clone() else {
                return false;
            };
            if !target.get_entity().is_alive() {
                return false;
            }

            let breeze_pos = entity.pos.load();
            let target_pos = target.get_entity().pos.load();
            let follow_range = breeze
                .mob_entity
                .living_entity
                .get_attribute_value(&Attributes::FOLLOW_RANGE);

            if Self::out_of_aggro_range(breeze_pos, target_pos, follow_range) {
                *breeze.mob_entity.target.lock().await = None;
                return false;
            }
            if Self::too_close_for_jump(breeze_pos, target_pos) {
                return false;
            }
            if !Self::can_jump_from_current_position(mob) {
                return false;
            }

            let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()));
            let raw_target = random_point_behind_target(
                target_pos,
                target.get_entity().head_yaw.load(),
                &mut random,
            );
            let Some(landing) = Self::snap_to_surface(mob, raw_target).await else {
                return false;
            };

            let landing_center = landing.to_f64().add_raw(0.5, 0.0, 0.5);
            let landing_above = landing_center.add_raw(0.0, 4.0, 0.0);
            if !Self::has_line_of_sight(mob, landing_center).await
                && !Self::has_line_of_sight(mob, landing_above).await
            {
                return false;
            }

            self.jump_target = Some(landing);
            true
        })
    }

    fn should_continue<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { self.phase != Phase::Idle })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.phase = Phase::Inhaling {
                ticks_left: INHALING_DURATION_TICKS,
            };
            self.leaving_water = false;
            let entity = mob.get_entity();
            entity.set_pose(EntityPose::Inhaling);
            let pos = entity.pos.load();
            entity
                .world
                .load()
                .play_sound(Sound::EntityBreezeCharge, SoundCategory::Hostile, &pos);

            if let Some(target) = self.jump_target {
                mob.get_mob_entity()
                    .look_control
                    .lock()
                    .unwrap()
                    .look_at_position(mob, target.to_f64().add_raw(0.5, 0.5, 0.5));
            }
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let entity = mob.get_entity();
            if entity.pose.load() == EntityPose::LongJumping
                || entity.pose.load() == EntityPose::Inhaling
            {
                entity.set_pose(EntityPose::Standing);
            }
            self.phase = Phase::Idle;
            self.jump_target = None;
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(breeze) = self.breeze.upgrade() else {
                return;
            };
            let entity = mob.get_entity();
            let in_water = entity.touching_water.load(Relaxed);
            if !in_water && self.leaving_water {
                self.leaving_water = false;
            }

            match self.phase {
                Phase::Inhaling { ticks_left } => {
                    if ticks_left > 1 {
                        self.phase = Phase::Inhaling {
                            ticks_left: ticks_left - 1,
                        };
                        return;
                    }

                    let Some(target_block) = self.jump_target else {
                        entity.set_pose(EntityPose::Standing);
                        self.phase = Phase::Idle;
                        return;
                    };

                    let breeze_pos = entity.pos.load();
                    // `Vec3.atBottomCenterOf`: block-center X/Z, floor Y - distinct from
                    // the `atCenterOf` point used for the LOS check in `can_start`.
                    let target_pos = target_block.to_f64().add_raw(0.5, 0.0, 0.5);
                    let gravity = breeze.mob_entity.living_entity.get_gravity();
                    let follow_range = breeze
                        .mob_entity
                        .living_entity
                        .get_attribute_value(&Attributes::FOLLOW_RANGE);

                    let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()));
                    let Some(velocity) = Self::calculate_optimal_jump_vector(
                        breeze_pos,
                        target_pos,
                        gravity,
                        follow_range,
                        &mut random,
                    ) else {
                        entity.set_pose(EntityPose::Standing);
                        self.phase = Phase::Idle;
                        return;
                    };

                    if in_water {
                        self.leaving_water = true;
                    }

                    entity.world.load().play_sound(
                        Sound::EntityBreezeJump,
                        SoundCategory::Hostile,
                        &breeze_pos,
                    );
                    entity.set_pose(EntityPose::LongJumping);
                    entity.yaw.store(entity.head_yaw.load());
                    entity.velocity.store(velocity);
                    // Vanilla also calls `setDiscardFriction(true)` here to skip normal
                    // ground/air drag mid-arc; Pumpkin's movement pipeline has no
                    // equivalent hook, so the jump loses a little more speed to drag
                    // than vanilla's.
                    self.phase = Phase::Jumping;
                }
                Phase::Jumping => {
                    let landed_on_ground = entity.on_ground.load(Relaxed);
                    let landed_in_water = in_water && !self.leaving_water;
                    if landed_on_ground || landed_in_water {
                        let breeze_pos = entity.pos.load();
                        entity.world.load().play_sound(
                            Sound::EntityBreezeLand,
                            SoundCategory::Hostile,
                            &breeze_pos,
                        );
                        entity.set_pose(EntityPose::Standing);

                        let living = &breeze.mob_entity.living_entity;
                        let recently_hurt = living.entity.age.load(Relaxed)
                            - living.last_attacked_time.load(Relaxed)
                            < RECENT_HURT_TICKS;
                        breeze.set_jump_cooldown(if recently_hurt {
                            JUMP_COOLDOWN_WHEN_HURT_TICKS
                        } else {
                            JUMP_COOLDOWN_TICKS
                        });
                        breeze.set_shoot_window(100);
                        self.phase = Phase::Idle;
                    }
                }
                Phase::Idle => {}
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK | Controls::JUMP
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_short_hop_has_a_valid_solution() {
        let mob_pos = Vector3::new(0.0, 64.0, 0.0);
        let target_pos = Vector3::new(6.0, 64.0, 0.0);
        let v = calculate_jump_vector_for_angle(mob_pos, target_pos, 0.08, 1.4, 55);
        assert!(v.is_some());
    }

    #[test]
    fn velocity_above_max_is_rejected() {
        let mob_pos = Vector3::new(0.0, 64.0, 0.0);
        let target_pos = Vector3::new(50.0, 64.0, 0.0);
        // Max velocity far too small to cover 50 blocks.
        let v = calculate_jump_vector_for_angle(mob_pos, target_pos, 0.08, 0.1, 45);
        assert!(v.is_none());
    }

    #[test]
    fn negative_discriminant_is_rejected() {
        // A steep angle jumping to a point far above the mob can produce a negative
        // v0^2 (no real launch speed reaches it at that angle).
        let mob_pos = Vector3::new(0.0, 0.0, 0.0);
        let target_pos = Vector3::new(1.0, 100.0, 0.0);
        let v = calculate_jump_vector_for_angle(mob_pos, target_pos, 0.08, 1.4, 80);
        assert!(v.is_none());
    }

    #[test]
    fn max_jump_velocity_matches_vanilla_default_follow_range() {
        // Breeze.createAttributes: FOLLOW_RANGE = 24.0 -> maxJumpVelocity = 1.4 (DEFAULT_MAX_JUMP_VELOCITY).
        let max_v = MAX_JUMP_VELOCITY_MULTIPLIER * 24.0;
        assert!((max_v - 1.4).abs() < 1.0e-6);
    }

    #[test]
    fn too_close_for_jump_matches_the_four_block_threshold() {
        let breeze_pos = Vector3::new(0.0, 0.0, 0.0);
        assert!(BreezeJumpGoal::too_close_for_jump(
            breeze_pos,
            Vector3::new(4.0, 0.0, 0.0)
        ));
        assert!(!BreezeJumpGoal::too_close_for_jump(
            breeze_pos,
            Vector3::new(4.01, 0.0, 0.0)
        ));
    }
}
