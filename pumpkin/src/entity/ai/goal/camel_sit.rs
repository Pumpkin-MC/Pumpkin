use std::sync::atomic::Ordering::Relaxed;

use super::{Controls, Goal, GoalFuture};
use crate::entity::mob::Mob;
use pumpkin_data::sound::{Sound, SoundCategory};
use rand::RngExt;

/// Vanilla: `CamelAi.RandomSitting.minimalPoseTicks` (`minimalPoseTimeSec = 20`). The camel must
/// have been standing (or sitting) for at least this long before it's willing to change pose
/// again.
const MIN_POSE_TICKS: i32 = 20 * 20;
/// Chance per eligible tick of toggling pose, once `MIN_POSE_TICKS` has elapsed. Vanilla instead
/// picks `CamelAi.RandomSitting` as one weighted option out of a `RunOne` behavior (itself only
/// re-rolled a few times a second); this constant is a from-scratch approximation tuned to
/// produce a similar "occasionally sits down for a while" cadence, not a ported vanilla value.
const SIT_CHANCE_PER_TICK: f32 = 1.0 / 600.0;
const STAND_CHANCE_PER_TICK: f32 = 1.0 / 600.0;

/// Makes an idle camel occasionally sit down for a while, then stand back up.
///
/// This is a simplified port of vanilla's `CamelAi.RandomSitting` behavior. Notable
/// simplifications:
/// - Vanilla's actual dash mechanic (`Camel.dash`) is entirely player-input driven (a
///   saddled, ridden camel dashes when its controlling passenger double-taps jump) rather than
///   AI-goal driven, and Pumpkin does not yet have camel-riding/jump-dash input plumbing, so it
///   is out of scope for this `Goal` -- only the idle sit/stand behavior is implemented here.
/// - No sitting pose/animation sync (`Pose.SITTING`, `AnimationState`s, hitbox resize to
///   `ADULT_SITTING_DIMENSIONS`) since Pumpkin does not track camel-specific entity metadata;
///   the camel freezes in place but renders standing, same caveat as `SitGoal`
///   (`pumpkin/src/entity/ai/goal/sit.rs`).
/// - No leash/passenger/water checks from `RandomSitting.checkExtraStartConditions`; only
///   `on_ground` is checked.
/// - Sit/stand toggling uses a flat per-tick probability instead of porting vanilla's `RunOne`
///   weighted-choice selection (see `SIT_CHANCE_PER_TICK`).
///
/// Vanilla source: `net/minecraft/world/entity/animal/camel/CamelAi.java` (`RandomSitting`),
/// `net/minecraft/world/entity/animal/camel/Camel.java` (`sitDown`/`standUp`).
pub struct CamelSitGoal {
    goal_control: Controls,
    pose_ticks: i32,
}

impl CamelSitGoal {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            goal_control: Controls::MOVE,
            pose_ticks: 0,
        }
    }
}

impl Default for CamelSitGoal {
    fn default() -> Self {
        Self::new()
    }
}

impl Goal for CamelSitGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            self.pose_ticks += 1;
            if self.pose_ticks < MIN_POSE_TICKS {
                return false;
            }

            let entity = mob.get_entity();
            if !entity.on_ground.load(Relaxed) || entity.touching_water.load(Relaxed) {
                return false;
            }

            mob.get_random().random::<f32>() < SIT_CHANCE_PER_TICK
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if self.pose_ticks < MIN_POSE_TICKS {
                return true;
            }
            mob.get_random().random::<f32>() >= STAND_CHANCE_PER_TICK
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.pose_ticks = 0;
            mob.get_mob_entity().navigator.lock().unwrap().stop();

            let entity = mob.get_entity();
            let world = entity.world.load();
            world.play_sound(
                Sound::EntityCamelSit,
                SoundCategory::Neutral,
                &entity.pos.load(),
            );
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.pose_ticks = 0;

            let entity = mob.get_entity();
            let world = entity.world.load();
            world.play_sound(
                Sound::EntityCamelStand,
                SoundCategory::Neutral,
                &entity.pos.load(),
            );
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.pose_ticks += 1;
            // Keep the camel from wandering off while sitting.
            mob.get_mob_entity().navigator.lock().unwrap().stop();
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
