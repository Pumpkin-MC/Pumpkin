//! Port of `behavior/MoveToTargetSink.java`.
//!
//! This is the load-bearing bridge between brain memory and the movement stack: it is the only
//! reader of `WALK_TARGET`, and every behavior that wants the mob to go somewhere does it by
//! writing that memory and letting this sink drain it into `Navigator`.
//!
//! # Deviations from vanilla, and why
//!
//! Vanilla's version is built on `PathNavigation.createPath(pos, 0)` returning a `Path` object
//! that the behavior stores, tests with `canReach()`, publishes into `MemoryModuleType.PATH`,
//! and finally hands to `moveTo(path, speed)` (`MoveToTargetSink.java:92-93,113-140`).
//! Pumpkin's `Navigator` has no such API: the only entry point is
//! `set_progress(NavigatorGoal::new(from, to, speed))` (`ai/pathfinder/mod.rs:101-105`), and the
//! path is computed lazily inside `Navigator::tick` (`ai/pathfinder/mod.rs:339-345`). So:
//!
//! - `MemoryModuleType.PATH` is not represented, and is dropped from the entry condition.
//!   Vanilla uses `PATH` `VALUE_ABSENT` to stop the sink restarting while a path is live; the
//!   replacement guard is `Navigator::is_idle()` in `can_still_use`.
//! - `CANT_REACH_WALK_TARGET_SINCE` stays in the entry condition as `REGISTERED` and is erased on
//!   reaching the target, but is never *set*: it is only written from vanilla's
//!   `path.canReach()` branch, and Pumpkin cannot answer that question before the navigator has
//!   run. Nothing in this stage reads it.
//! - The `DefaultRandomPos.getPosTowards` partial-step fallback (`:132-136`) is not ported --
//!   Pumpkin's navigator already degrades to a best-effort partial path internally
//!   (`ai/pathfinder/mod.rs:266-292` returns the best node reached even when `reached == false`).
//! - `stop`'s `remainingCooldown = random.nextInt(40)` is gated on
//!   `body.getNavigation().isStuck()` (`:79-83`), which Pumpkin's `Navigator` does not expose.
//!   The cooldown field and its `checkExtraStartConditions` drain are kept so the shape is
//!   right, but nothing currently sets it.

use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::ai::brain::Brain;
use crate::entity::ai::brain::behavior::{Behavior, TimedBehavior, TimedBehaviorControl};
use crate::entity::ai::brain::memory::{
    CantReachWalkTargetSinceMemory, MemoryKeyId, MemoryStatus, WalkTarget, WalkTargetMemory,
};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;

pub struct MoveToTargetSink {
    /// `MoveToTargetSink.remainingCooldown` (`:21`).
    remaining_cooldown: i32,
    /// `MoveToTargetSink.lastTargetPos` (`:23`), used to decide when the target has drifted far
    /// enough to be worth re-pathing.
    last_target_pos: Option<BlockPos>,
    last_target_position: Option<Vector3<f64>>,
    /// `MoveToTargetSink.speedModifier` (`:24`). A *modifier*, matching
    /// `WalkTarget.getSpeedModifier()` and `NavigatorGoal.speed`, which
    /// `Navigator::tick` feeds to `LivingEntity::speed_for_modifier`
    /// (`ai/pathfinder/mod.rs:434`, `entity/living.rs:570`).
    speed_modifier: f32,
}

impl MoveToTargetSink {
    /// `new MoveToTargetSink()` -- `this(150, 250)` (`MoveToTargetSink.java:26-28`).
    // Returns a boxed trait object, not Self by name -- constructor pattern for this behavior/sensor family.
    #[allow(clippy::new_ret_no_self)]
    #[must_use]
    pub fn new() -> Box<dyn Behavior> {
        Self::with_timeout(150, 250)
    }

    #[must_use]
    pub fn with_timeout(min_timeout: i32, max_timeout: i32) -> Box<dyn Behavior> {
        Box::new(TimedBehaviorControl::with_duration(
            Self {
                remaining_cooldown: 0,
                last_target_pos: None,
                last_target_position: None,
                speed_modifier: 1.0,
            },
            vec![
                (
                    MemoryKeyId::CantReachWalkTargetSince,
                    MemoryStatus::Registered,
                ),
                (MemoryKeyId::WalkTarget, MemoryStatus::ValuePresent),
            ],
            min_timeout,
            max_timeout,
        ))
    }

    /// `reachedTarget` (`MoveToTargetSink.java:142-144`). Manhattan distance, not Euclidean.
    fn reached_target(mob: &dyn Mob, walk_target: &WalkTarget) -> bool {
        let Some(target) = walk_target.target.current_block_position() else {
            return false;
        };
        let mob_pos = mob.get_mob_entity().living_entity.entity.block_pos.load();
        let distance = (target.0.x - mob_pos.0.x).abs()
            + (target.0.y - mob_pos.0.y).abs()
            + (target.0.z - mob_pos.0.z).abs();
        distance <= walk_target.close_enough_dist
    }

    /// Hands the current target to the navigator. Split out because vanilla's `tick` re-invokes
    /// `start` on target drift (`MoveToTargetSink.java:106-109`).
    ///
    /// The memory lock is NOT held here: the caller reads `WALK_TARGET` into a local first, then
    /// this takes the navigator lock. Never invert that order.
    fn push_to_navigator(&self, mob: &dyn Mob) {
        let Some(destination) = self.last_target_position else {
            return;
        };
        let mob_entity = mob.get_mob_entity();
        let current = mob_entity.living_entity.entity.pos.load();
        let mut navigator = mob_entity.navigator.lock().unwrap();
        navigator.set_progress(NavigatorGoal::new(
            current,
            destination,
            f64::from(self.speed_modifier),
        ));
    }
}

impl TimedBehavior for MoveToTargetSink {
    fn debug_name(&self) -> &'static str {
        "MoveToTargetSink"
    }

    /// `checkExtraStartConditions` (`MoveToTargetSink.java:45-65`).
    fn check_extra_start_conditions(&mut self, mob: &dyn Mob, brain: &Brain) -> bool {
        if self.remaining_cooldown > 0 {
            self.remaining_cooldown -= 1;
            return false;
        }

        let Some(walk_target) = brain.get::<WalkTargetMemory>() else {
            return false;
        };

        let reached = Self::reached_target(mob, &walk_target);
        if !reached
            && let Some(block_pos) = walk_target.target.current_block_position()
            && let Some(position) = walk_target.target.current_position()
        {
            self.speed_modifier = walk_target.speed_modifier;
            self.last_target_pos = Some(block_pos);
            self.last_target_position = Some(position);
            return true;
        }

        brain.erase::<WalkTargetMemory>();
        if reached {
            brain.erase::<CantReachWalkTargetSinceMemory>();
        }
        false
    }

    /// `start` (`MoveToTargetSink.java:91-94`): vanilla publishes the path into memory and calls
    /// `navigation.moveTo(path, speed)`. Here there is no path object, so the destination goes
    /// straight to the navigator.
    fn start(&mut self, mob: &dyn Mob, _brain: &Brain, _game_time: i64) {
        self.push_to_navigator(mob);
    }

    /// `canStillUse` (`MoveToTargetSink.java:67-76`). `!navigation.isDone()` becomes
    /// `!Navigator::is_idle()`, which also stands in for vanilla's `PATH`-absent entry guard.
    /// The spectator check (`isWalkTargetSpectator`) is dropped: it needs
    /// `EntityTracker`-typed introspection of the walk target plus a spectator predicate.
    fn can_still_use(&mut self, mob: &dyn Mob, brain: &Brain, _game_time: i64) -> bool {
        if self.last_target_pos.is_none() {
            return false;
        }
        let Some(walk_target) = brain.get::<WalkTargetMemory>() else {
            return false;
        };
        if Self::reached_target(mob, &walk_target) {
            return false;
        }
        !mob.get_mob_entity().navigator.lock().unwrap().is_idle()
    }

    /// `tick` (`MoveToTargetSink.java:96-111`): re-path only once the target has drifted more
    /// than 4 blocks squared from where it was when the path was computed. Without this guard
    /// `Navigator::set_progress` would clear `current_path` every tick
    /// (`ai/pathfinder/mod.rs:104`) and the mob would stutter in place.
    fn tick(&mut self, mob: &dyn Mob, brain: &Brain, _game_time: i64) {
        let Some(last) = self.last_target_pos else {
            return;
        };
        let Some(walk_target) = brain.get::<WalkTargetMemory>() else {
            return;
        };
        let Some(current_block) = walk_target.target.current_block_position() else {
            return;
        };

        let dx = f64::from(current_block.0.x - last.0.x);
        let dy = f64::from(current_block.0.y - last.0.y);
        let dz = f64::from(current_block.0.z - last.0.z);
        if dx * dx + dy * dy + dz * dz <= 4.0 {
            return;
        }

        let Some(position) = walk_target.target.current_position() else {
            return;
        };
        self.speed_modifier = walk_target.speed_modifier;
        self.last_target_pos = Some(current_block);
        self.last_target_position = Some(position);
        self.push_to_navigator(mob);
    }

    /// `stop` (`MoveToTargetSink.java:78-89`).
    fn stop(&mut self, mob: &dyn Mob, brain: &Brain, _game_time: i64) {
        mob.get_mob_entity().navigator.lock().unwrap().stop();
        brain.erase::<WalkTargetMemory>();
        self.last_target_pos = None;
        self.last_target_position = None;
    }
}
