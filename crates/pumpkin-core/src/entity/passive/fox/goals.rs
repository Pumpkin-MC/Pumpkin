//! Vanilla `Fox` inner goals.

use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;

use pumpkin_data::Block;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::Sound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::rot_lerp;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use super::{FoxEntity, as_fox};
use crate::entity::EntityBase;
use crate::entity::ai::goal::melee_attack::MeleeAttackGoal;
use crate::entity::ai::goal::track_target::TrackTargetGoal;
use crate::entity::ai::goal::{Controls, Goal, GoalReach, to_goal_ticks};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::mob::Mob;
use crate::entity::mob::neutral::find_by_uuid;
use crate::plugin::api::events::entity::entity_target::TargetReason;

type FoxCheck = fn(&FoxEntity) -> bool;

/// A shared goal with the checks and start step of vanilla's fox subclass.
pub struct FoxGoal<G> {
    inner: G,
    start_if: Option<FoxCheck>,
    continue_if: Option<FoxCheck>,
    on_start: Option<fn(&FoxEntity)>,
}

impl<G: Goal> FoxGoal<G> {
    pub const fn new(inner: G) -> Self {
        Self {
            inner,
            start_if: None,
            continue_if: None,
            on_start: None,
        }
    }

    /// Extra `canUse` check.
    pub const fn start_if(mut self, check: FoxCheck) -> Self {
        self.start_if = Some(check);
        self
    }

    /// Extra `canUse` and `canContinueToUse` check.
    pub const fn gated(mut self, check: FoxCheck) -> Self {
        self.start_if = Some(check);
        self.continue_if = Some(check);
        self
    }

    pub const fn on_start(mut self, step: fn(&FoxEntity)) -> Self {
        self.on_start = Some(step);
        self
    }

    fn passes(mob: &dyn Mob, check: Option<FoxCheck>) -> bool {
        check.is_none_or(|check| as_fox(mob).is_some_and(check))
    }
}

impl<G: Goal> Goal for FoxGoal<G> {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        Self::passes(mob, self.start_if) && self.inner.can_start(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        Self::passes(mob, self.continue_if) && self.inner.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(step) = self.on_start
            && let Some(fox) = as_fox(mob)
        {
            step(fox);
        }
        self.inner.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.inner.stop(mob);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.inner.tick(mob);
    }

    fn should_run_every_tick(&self) -> bool {
        self.inner.should_run_every_tick()
    }

    fn can_stop(&self) -> bool {
        self.inner.can_stop()
    }

    fn controls(&self) -> Controls {
        self.inner.controls()
    }

    /// The fox checks only read the fox itself.
    fn tick_reach(&self, mob: &dyn Mob, running: bool) -> Option<GoalReach> {
        self.inner.tick_reach(mob, running)
    }
}

/// Vanilla `FoxMeleeAttackGoal`: bites without a swing, never from a sitting or crouched pose.
pub fn melee_attack(speed: f64) -> FoxGoal<MeleeAttackGoal> {
    FoxGoal::new(MeleeAttackGoal::new(speed, true).performing(bite))
        .start_if(|fox| {
            !fox.is_sitting() && !fox.is_sleeping() && !fox.is_crouching() && !fox.is_faceplanted()
        })
        .on_start(|fox| fox.set_interested(false))
}

fn bite(mob: &dyn Mob, target: &dyn EntityBase) {
    if mob.get_mob_entity().try_attack(mob.get_entity(), target) {
        mob.on_attack(target);
    }
    mob.get_entity().play_sound(Sound::EntityFoxBite);
}

fn look_at(mob: &dyn Mob, target: &Arc<dyn EntityBase>, max_yaw: f32, max_pitch: f32) {
    mob.get_mob_entity()
        .look_control
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .look_at_entity_with_range(target, max_yaw, max_pitch);
}

fn stop_navigation(mob: &dyn Mob) {
    mob.get_mob_entity().steered().navigator().stop();
}

fn living_target(mob: &dyn Mob) -> Option<Arc<dyn EntityBase>> {
    mob.get_mob_entity()
        .get_target()
        .filter(|target| target.get_entity().is_alive())
}

/// Vanilla `Fox.isPathClear`: room to leap at six points along the way.
fn is_path_clear(mob: &dyn Mob, target: &dyn EntityBase) -> bool {
    let entity = mob.get_entity();
    let fox = entity.pos.load();
    let target = target.get_entity().pos.load();
    let zdiff = target.z - fox.z;
    let xdiff = target.x - fox.x;
    let slope = zdiff / xdiff;
    let world = entity.world.load();
    (0..6u8).all(|i| {
        let step = f64::from(f32::from(i) / 6.0);
        let z = if slope == 0.0 { 0.0 } else { zdiff * step };
        let x = if slope == 0.0 {
            xdiff * step
        } else {
            z / slope
        };
        (1..4u8).all(|j| {
            let pos = BlockPos::floored(fox.x + x, fox.y + f64::from(j), fox.z + z);
            world.get_block_state(&pos).replaceable()
        })
    })
}

/// Vanilla `DefendTrustedTargetGoal`: attacks whatever last hurt the first trusted entity found.
pub struct DefendTrustedTargetGoal {
    track_target_goal: TrackTargetGoal,
    target_predicate: TargetPredicate,
    trusted: Option<Arc<dyn EntityBase>>,
    attacker: Option<Arc<dyn EntityBase>>,
    timestamp: i32,
}

impl DefendTrustedTargetGoal {
    #[must_use]
    pub fn new() -> Self {
        Self {
            track_target_goal: TrackTargetGoal::with_default(false),
            target_predicate: TargetPredicate::create_attackable(),
            trusted: None,
            attacker: None,
            timestamp: 0,
        }
    }

    /// Vanilla `TRUSTED_TARGET_SELECTOR` and the untrusted check.
    fn is_valid_attacker(fox: &FoxEntity, attacker: &dyn EntityBase) -> bool {
        attacker.get_living_entity().is_some_and(|living| {
            living.last_attacking_id.load(Relaxed) != 0
                && living.last_attack_time.load(Relaxed)
                    < living.entity.tick_count.load(Relaxed) + 600
        }) && !fox.trusts(attacker.get_entity().entity_uuid)
    }
}

impl Goal for DefendTrustedTargetGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if mob.get_random().random_range(0..to_goal_ticks(10)) != 0 {
            return false;
        }
        let Some(fox) = as_fox(mob) else {
            return false;
        };
        let world = mob.get_entity().world.load();
        let Some(trusted) = fox.trusted.load().into_iter().flatten().find_map(|uuid| {
            find_by_uuid(&world, uuid).filter(|entity| entity.get_living_entity().is_some())
        }) else {
            return false;
        };
        let Some(living) = trusted.get_living_entity() else {
            return false;
        };
        let timestamp = living.get_last_hurt_by_mob_timestamp();
        let attacker = living.get_last_hurt_by_mob();
        self.trusted = Some(trusted.clone());
        self.attacker.clone_from(&attacker);
        if timestamp == self.timestamp {
            return false;
        }
        let Some(attacker) = attacker else {
            return false;
        };
        self.target_predicate.base_max_distance =
            self.track_target_goal.follow_range(mob.get_mob_entity());
        Self::is_valid_attacker(fox, attacker.as_ref())
            && self.track_target_goal.can_track(
                mob,
                Some(attacker.as_ref()),
                &self.target_predicate,
            )
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.track_target_goal.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.track_target_goal.start_targeting(
            mob,
            self.attacker.clone(),
            TargetReason::TargetAttackedOwner,
        );
        if let Some(living) = self.trusted.as_ref().and_then(|t| t.get_living_entity()) {
            self.timestamp = living.get_last_hurt_by_mob_timestamp();
        }
        if let Some(fox) = as_fox(mob) {
            fox.get_entity().play_sound(Sound::EntityFoxAggro);
            fox.set_defending(true);
            fox.set_sleeping(false);
        }
        self.track_target_goal.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.trusted = None;
        self.attacker = None;
        self.track_target_goal.stop(mob);
    }

    fn controls(&self) -> Controls {
        self.track_target_goal.controls()
    }
}

/// Vanilla `FaceplantGoal`: stuck head first in snow for 40 ticks.
#[derive(Default)]
pub struct FaceplantGoal {
    countdown: i32,
}

impl Goal for FaceplantGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        as_fox(mob).is_some_and(FoxEntity::is_faceplanted)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.can_start(mob) && self.countdown > 0
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.countdown = self.get_tick_count(40);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        if let Some(fox) = as_fox(mob) {
            fox.set_faceplanted(false);
        }
    }

    fn tick(&mut self, _mob: &dyn Mob) {
        self.countdown -= 1;
    }

    fn controls(&self) -> Controls {
        Controls::LOOK | Controls::JUMP | Controls::MOVE
    }
}

/// Within this squared distance the fox stops and crouches.
const STALK_DISTANCE_SQ: f64 = 36.0;

/// Vanilla `StalkPreyGoal`: sneaks up on chickens and rabbits, then crouches to pounce.
#[derive(Default)]
pub struct StalkPreyGoal;

impl StalkPreyGoal {
    fn crouch(mob: &dyn Mob, fox: &FoxEntity) {
        fox.set_interested(true);
        fox.set_crouching(true);
        stop_navigation(mob);
    }
}

impl Goal for StalkPreyGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = as_fox(mob) else {
            return false;
        };
        if fox.is_sleeping() {
            return false;
        }
        let Some(target) = living_target(mob) else {
            return false;
        };
        let target_entity = target.get_entity();
        let prey = target_entity.entity_type == &EntityType::CHICKEN
            || target_entity.entity_type == &EntityType::RABBIT;
        prey && mob
            .get_entity()
            .pos
            .load()
            .squared_distance_to_vec(&target_entity.pos.load())
            > STALK_DISTANCE_SQ
            && !fox.is_crouching()
            && !fox.is_interested()
            && !mob.get_mob_entity().living_entity.jumping.load(Relaxed)
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(fox) = as_fox(mob) {
            fox.set_sitting(false);
            fox.set_faceplanted(false);
        }
    }

    fn stop(&mut self, mob: &dyn Mob) {
        let Some(fox) = as_fox(mob) else {
            return;
        };
        match mob.get_mob_entity().get_target() {
            Some(target) if is_path_clear(mob, target.as_ref()) => {
                Self::crouch(mob, fox);
                look_at(
                    mob,
                    &target,
                    mob.get_max_head_rotation(),
                    mob.get_max_look_pitch_change(),
                );
            }
            _ => {
                fox.set_interested(false);
                fox.set_crouching(false);
            }
        }
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let (Some(fox), Some(target)) = (as_fox(mob), mob.get_mob_entity().get_target()) else {
            return;
        };
        look_at(
            mob,
            &target,
            mob.get_max_head_rotation(),
            mob.get_max_look_pitch_change(),
        );
        let pos = mob.get_entity().pos.load();
        let target_pos = target.get_entity().pos.load();
        if pos.squared_distance_to_vec(&target_pos) <= STALK_DISTANCE_SQ {
            Self::crouch(mob, fox);
        } else {
            mob.get_mob_entity()
                .steered()
                .navigator()
                .set_progress(NavigatorGoal::new(pos, target_pos, 1.5));
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}

/// Java `Math.signum`, which keeps zero.
fn signum(value: f64) -> f64 {
    if value == 0.0 { value } else { value.signum() }
}

/// Vanilla `FoxPounceGoal`: the leap from a full crouch, nose first into snow when it lands there.
#[derive(Default)]
pub struct PounceGoal;

impl PounceGoal {
    /// Pitch follows the leap's arc, then levels out once it is flat.
    fn tilt(mob: &dyn Mob) {
        let entity = mob.get_entity();
        let movement = entity.velocity.load();
        let pitch = entity.pitch.load();
        if movement.y * movement.y < f64::from(0.03f32) && pitch != 0.0 {
            entity.set_pitch(rot_lerp(0.2, pitch, 0.0));
            return;
        }
        let direction = movement.horizontal_length();
        let jumping = mob.get_mob_entity().living_entity.jumping.load(Relaxed);
        let upwards_bias = if jumping && movement.y > 0.0 {
            6.5
        } else {
            1.0
        };
        let biased_y = movement.y * upwards_bias;
        let len = direction.hypot(biased_y);
        if len > f64::from(1.0e-5f32) {
            let rotation = signum(-biased_y) * (direction / len).acos() * f64::from(180.0f32)
                / f64::from(std::f32::consts::PI);
            entity.set_pitch(rotation as f32);
        }
    }
}

impl Goal for PounceGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = as_fox(mob) else {
            return false;
        };
        if !fox.is_fully_crouched() {
            return false;
        }
        // Vanilla also compares the target's motion and facing direction, always equal for mobs.
        let Some(target) = living_target(mob) else {
            return false;
        };
        let clear = is_path_clear(mob, target.as_ref());
        if !clear {
            fox.set_crouching(false);
            fox.set_interested(false);
        }
        clear
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = as_fox(mob) else {
            return false;
        };
        if living_target(mob).is_none() {
            return false;
        }
        let entity = mob.get_entity();
        let dy = entity.velocity.load().y;
        let landed = dy * dy < f64::from(0.05f32)
            && entity.pitch.load().abs() < 15.0
            && entity.on_ground.load(Relaxed);
        !landed && !fox.is_faceplanted()
    }

    fn can_stop(&self) -> bool {
        false
    }

    fn start(&mut self, mob: &dyn Mob) {
        // Vanilla's `setJumping(true)` is reset by the jump control before it acts.
        if let Some(fox) = as_fox(mob) {
            fox.set_pouncing(true);
            fox.set_interested(false);
        }
        if let Some(target) = mob.get_mob_entity().get_target() {
            look_at(mob, &target, 60.0, 30.0);
            let entity = mob.get_entity();
            let offset = target.get_entity().pos.load() - entity.pos.load();
            let length = offset.length();
            let direction = if length < f64::from(1.0e-5f32) {
                Vector3::default()
            } else {
                offset / length
            };
            entity.set_velocity(
                entity.velocity.load() + Vector3::new(direction.x * 0.8, 0.9, direction.z * 0.8),
            );
        }
        stop_navigation(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        if let Some(fox) = as_fox(mob) {
            fox.set_crouching(false);
            fox.crouch_amount.store(0.0);
            fox.set_interested(false);
            fox.set_pouncing(false);
        }
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(fox) = as_fox(mob) else {
            return;
        };
        let target = mob.get_mob_entity().get_target();
        if let Some(target) = &target {
            look_at(mob, target, 60.0, 30.0);
        }
        if !fox.is_faceplanted() {
            Self::tilt(mob);
        }

        let entity = mob.get_entity();
        if let Some(target) = target {
            let offset = entity.pos.load() - target.get_entity().pos.load();
            let (x, y, z) = (offset.x as f32, offset.y as f32, offset.z as f32);
            if (x * x + y * y + z * z).sqrt() <= 2.0 {
                if mob.get_mob_entity().try_attack(entity, target.as_ref()) {
                    mob.on_attack(target.as_ref());
                }
                return;
            }
        }
        let world = entity.world.load();
        if entity.pitch.load() > 0.0
            && entity.on_ground.load(Relaxed)
            && entity.velocity.load().y as f32 != 0.0
            && world.get_block(&entity.block_pos.load()).id == Block::SNOW.id
        {
            entity.set_pitch(60.0);
            mob.set_mob_target(None, TargetReason::Unknown);
            fox.set_faceplanted(true);
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::JUMP
    }
}
