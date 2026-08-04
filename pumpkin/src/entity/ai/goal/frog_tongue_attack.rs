//! Frog tongue-attack behavior.
//!
//! Vanilla source: `net/minecraft/world/entity/animal/frog/ShootTongue.java` (the two-phase
//! "extend tongue, catch, eat" behavior) and `Frog::canEat` (target filtering: entity must be
//! tagged `minecraft:frog_food` -- slime or magma cube -- and, for cube mobs, must be the
//! smallest size, i.e. `AbstractCubeMob::getSize() == 1`).
//!
//! Vanilla implements this with the Brain/Behavior/Memory framework; Pumpkin has no such
//! framework, so this is ported as two standalone `Goal`s: [`FrogFindFoodGoal`] (target
//! selector, mirrors `SensorType.FROG_ATTACKABLES`) and [`FrogTongueAttackGoal`] (goal
//! selector, mirrors the `ShootTongue` behavior).

use std::sync::Arc;

use pumpkin_data::entity::{EntityPose, EntityType};
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_protocol::codec::optional_int::OptionalInt;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::ai::goal::{Controls, Goal, GoalFuture};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::entity::mob::magma_cube::MagmaCubeEntity;
use crate::entity::mob::slime::SlimeEntity;
use crate::entity::{Entity, EntityBase};

/// Syncs the tracked-data `POSE` without going through `Entity::set_pose`'s per-pose
/// bounding-box resize. `Entity::get_entity_dimensions` has no frog-sized entry for
/// `UsingTongue`/`Standing`, so `set_pose` would fall back to its generic default (0.6x1.8,
/// player-sized) and distort the frog's hitbox for the animation's duration -- and could even
/// silently skip the pose change via its `is_space_empty` guard in tight spots. Vanilla's `Frog`
/// doesn't override `getDimensions` either, so its hitbox is pose-independent; this mirrors that
/// by only updating the pose field and the client-visible metadata.
fn sync_tongue_pose(entity: &Entity, pose: EntityPose) {
    entity.pose.store(pose);
    entity.send_meta_data(
        &[Metadata::new(
            TrackedData::POSE,
            MetaDataType::ENTITY_POSE,
            VarInt(pose as i32),
        )],
        None,
    );
}

/// Syncs `DATA_TONGUE_TARGET_ID` (`Frog::setTongueTarget`/`eraseTongueTarget`): tells clients
/// which entity to aim the tongue-extend animation at. Vanilla refreshes this every tick the
/// behavior runs and clears it on `stop`.
fn sync_tongue_target(entity: &Entity, target_id: Option<i32>) {
    entity.send_meta_data(
        &[Metadata::new(
            TrackedData::TONGUE_TARGET_ID,
            MetaDataType::OPTIONAL_UNSIGNED_INT,
            OptionalInt(target_id),
        )],
        None,
    );
}

/// Frog food candidates per `data/minecraft/tags/entity_type/frog_food.json`.
const FROG_FOOD_TYPES: &[&EntityType] = &[&EntityType::SLIME, &EntityType::MAGMA_CUBE];

/// Vanilla `Frog::canEat`: entity must be tagged `frog_food`, and cube mobs (slime/magma cube)
/// must additionally be the smallest size variant.
fn is_frog_food(entity: &dyn EntityBase) -> bool {
    if !FROG_FOOD_TYPES.contains(&entity.get_entity().entity_type) {
        return false;
    }
    if let Some(slime) = entity.cast_any().downcast_ref::<SlimeEntity>() {
        return slime.is_tiny();
    }
    if let Some(magma_cube) = entity.cast_any().downcast_ref::<MagmaCubeEntity>() {
        return magma_cube.slime.is_tiny();
    }
    false
}

/// Target-selector goal: finds the nearest edible slime/magma cube.
///
/// Vanilla drives this off `SensorType.FROG_ATTACKABLES`, which is continuously refreshed by a
/// brain sensor; here it's a normal polled target-selector `Goal` in the established style (see
/// `ActiveTargetGoal`).
pub struct FrogFindFoodGoal {
    range: f64,
}

impl FrogFindFoodGoal {
    #[must_use]
    pub const fn new(range: f64) -> Self {
        Self { range }
    }
}

impl Default for FrogFindFoodGoal {
    fn default() -> Self {
        // Vanilla `SensorType.FROG_ATTACKABLES` (`Sensor::TARGETING_RANGE`) uses 16 blocks.
        Self::new(16.0)
    }
}

impl Goal for FrogFindFoodGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let entity = mob.get_entity();
            let world = entity.world.load();
            let pos = entity.pos.load();

            // Vanilla covers this case via `StopAttackingIfTargetInvalid` /
            // `TrackTargetGoal::should_continue`; here it means clearing a stale target rather
            // than just refusing to pick a new one, otherwise a target that dies (or stops
            // qualifying) before `FrogTongueAttackGoal` starts would never be cleared and the
            // frog could never hunt again.
            {
                let mut target = mob.get_mob_entity().target.lock().await;
                if let Some(t) = target.as_ref()
                    && (!t.get_entity().is_alive() || !is_frog_food(t.as_ref()))
                {
                    *target = None;
                }
                if target.is_some() {
                    return false;
                }
            }

            let mut nearest: Option<(Arc<dyn EntityBase>, f64)> = None;
            for candidate in world.get_nearby_entities(pos, self.range).values() {
                if !candidate.get_entity().is_alive() || !is_frog_food(candidate.as_ref()) {
                    continue;
                }
                let dist_sq = candidate
                    .get_entity()
                    .pos
                    .load()
                    .squared_distance_to_vec(&pos);
                if nearest.as_ref().is_none_or(|(_, best)| dist_sq < *best) {
                    nearest = Some((candidate.clone(), dist_sq));
                }
            }

            if let Some((target, _)) = nearest {
                mob.set_mob_target(Some(target)).await;
                true
            } else {
                false
            }
        })
    }

    fn should_continue<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        // One-shot: hands off to `FrogTongueAttackGoal` immediately once a target is set.
        Box::pin(async { false })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum TongueState {
    MoveToTarget,
    CatchAnimation,
    EatAnimation,
    Done,
}

/// Vanilla `ShootTongue`: distance at which the tongue catches its target.
const EATING_DISTANCE: f64 = 1.75;
/// Vanilla `ShootTongue.CATCH_ANIMATION_DURATION`.
const CATCH_ANIMATION_DURATION: i32 = 6;
/// Vanilla `ShootTongue.TONGUE_ANIMATION_DURATION`.
const TONGUE_ANIMATION_DURATION: i32 = 10;
/// Vanilla `ShootTongue`: pathing is recalculated every 10 ticks while approaching.
const PATH_RECALC_INTERVAL: i32 = 10;
/// Vanilla `ShootTongue.EATING_MOVEMENT_FACTOR`: velocity imparted on the target when caught.
const EATING_MOVEMENT_FACTOR: f64 = 0.75;

/// Goal-selector goal: the tongue-attack itself, once a target has been assigned.
///
/// Vanilla source: `ShootTongue`. Scope-reduced: vanilla's `canPathfindToTarget` precheck and
/// the `UNREACHABLE_TONGUE_TARGETS` memory cooldown it feeds (skip a target the frog couldn't
/// path to for 100 ticks) are dropped, since Pumpkin's goal selector has no memory store to
/// mirror that cooldown into -- an unreachable target simply keeps being walked toward instead.
pub struct FrogTongueAttackGoal {
    goal_control: Controls,
    state: TongueState,
    timer: i32,
    path_recalc_counter: i32,
}

impl FrogTongueAttackGoal {
    #[must_use]
    pub fn new() -> Self {
        Self {
            goal_control: Controls::MOVE | Controls::LOOK,
            state: TongueState::Done,
            timer: 0,
            path_recalc_counter: 0,
        }
    }
}

impl Default for FrogTongueAttackGoal {
    fn default() -> Self {
        Self::new()
    }
}

impl Goal for FrogTongueAttackGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let target = mob.get_mob_entity().target.lock().await;
            let Some(target) = target.as_ref() else {
                return false;
            };
            target.get_entity().is_alive() && is_frog_food(target.as_ref())
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if self.state == TongueState::Done {
                return false;
            }
            let target = mob.get_mob_entity().target.lock().await;
            target.as_ref().is_some_and(|t| t.get_entity().is_alive())
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let target = mob.get_mob_entity().target.lock().await.clone();
            let Some(target) = target else {
                return;
            };
            mob.get_mob_entity()
                .look_control
                .lock()
                .unwrap()
                .look_at_entity_with_range(&target, 30.0, 30.0);
            sync_tongue_target(mob.get_entity(), Some(target.get_entity().entity_id));

            let mob_pos = mob.get_entity().pos.load();
            let target_pos = target.get_entity().pos.load();
            mob.get_mob_entity()
                .navigator
                .lock()
                .unwrap()
                .set_progress(NavigatorGoal {
                    current_progress: mob_pos,
                    destination: target_pos,
                    speed: 2.0,
                });

            self.state = TongueState::MoveToTarget;
            self.path_recalc_counter = PATH_RECALC_INTERVAL;
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            mob.set_mob_target(None).await;
            sync_tongue_pose(mob.get_entity(), EntityPose::Standing);
            sync_tongue_target(mob.get_entity(), None);
            mob.get_mob_entity().navigator.lock().unwrap().stop();
            self.state = TongueState::Done;
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let target = mob.get_mob_entity().target.lock().await.clone();
            let Some(target) = target else {
                self.state = TongueState::Done;
                return;
            };
            // Vanilla `ShootTongue::tick` re-sets `DATA_TONGUE_TARGET_ID` every tick, but
            // `SynchedEntityData` only broadcasts on an actual value change, and the target
            // can't change mid-run here -- so the `start()` broadcast already covers it and a
            // per-tick resend (which Pumpkin's `send_meta_data` would broadcast unconditionally)
            // is skipped as a no-op.

            match self.state {
                TongueState::MoveToTarget => {
                    let mob_pos = mob.get_entity().pos.load();
                    let target_pos = target.get_entity().pos.load();
                    let distance = mob_pos.squared_distance_to_vec(&target_pos).sqrt();

                    if distance < EATING_DISTANCE {
                        let world = mob.get_entity().world.load();
                        world.play_sound_fine(
                            Sound::EntityFrogTongue,
                            SoundCategory::Neutral,
                            &mob_pos,
                            2.0,
                            1.0,
                        );
                        sync_tongue_pose(mob.get_entity(), EntityPose::UsingTongue);
                        mob.get_mob_entity().navigator.lock().unwrap().stop();

                        let pull_dir = (mob_pos - target_pos).normalize() * EATING_MOVEMENT_FACTOR;
                        target
                            .get_entity()
                            .set_velocity(Vector3::new(pull_dir.x, pull_dir.y, pull_dir.z));

                        self.timer = 0;
                        self.state = TongueState::CatchAnimation;
                    } else {
                        self.path_recalc_counter -= 1;
                        if self.path_recalc_counter <= 0 {
                            mob.get_mob_entity().navigator.lock().unwrap().set_progress(
                                NavigatorGoal {
                                    current_progress: mob_pos,
                                    destination: target_pos,
                                    speed: 2.0,
                                },
                            );
                            self.path_recalc_counter = PATH_RECALC_INTERVAL;
                        }
                    }
                }
                TongueState::CatchAnimation | TongueState::EatAnimation => {
                    let (new_state, new_timer, should_eat) =
                        advance_catch_or_eat(self.state, self.timer);
                    if should_eat {
                        eat_target(mob, target.as_ref()).await;
                    }
                    self.state = new_state;
                    self.timer = new_timer;
                }
                TongueState::Done => {}
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

/// Advances the `CatchAnimation`/`EatAnimation` phases of the tongue timer, returning the next
/// state, the next timer value, and whether the target should be eaten this tick.
///
/// Vanilla: `eatAnimationTimer++ >= 6` -- the pre-increment value is compared, then the timer
/// always advances; it is not reset when moving into `EAT_ANIMATION`, so
/// `TONGUE_ANIMATION_DURATION` (10) is the total tongue animation length (~11 ticks from the
/// start of `CatchAnimation`), not a second countdown.
const fn advance_catch_or_eat(state: TongueState, timer: i32) -> (TongueState, i32, bool) {
    match state {
        TongueState::CatchAnimation => {
            if timer >= CATCH_ANIMATION_DURATION {
                (TongueState::EatAnimation, timer + 1, true)
            } else {
                (TongueState::CatchAnimation, timer + 1, false)
            }
        }
        TongueState::EatAnimation => {
            if timer >= TONGUE_ANIMATION_DURATION {
                (TongueState::Done, timer, false)
            } else {
                (TongueState::EatAnimation, timer + 1, false)
            }
        }
        other => (other, timer, false),
    }
}

/// Vanilla `ShootTongue::eatEntity`: attack the target for the frog's attack damage, then force
/// a silent removal (no drops) if it's somehow still alive -- vanilla's fallback for the case
/// where the attack alone doesn't kill it (`target.remove(Entity.RemovalReason.KILLED)`).
async fn eat_target(mob: &dyn Mob, target: &dyn EntityBase) {
    let world = mob.get_entity().world.load();
    world.play_sound_fine(
        Sound::EntityFrogEat,
        SoundCategory::Neutral,
        &mob.get_entity().pos.load(),
        2.0,
        1.0,
    );

    if !target.get_entity().is_alive() {
        return;
    }
    mob.try_attack(target).await;
    if target.get_entity().is_alive() {
        world.remove_entity(target).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eats_on_the_seventh_catch_tick_and_finishes_after_eleven() {
        let mut state = TongueState::CatchAnimation;
        let mut timer = 0;
        let mut ate_on_tick = None;

        for tick in 1..=20 {
            let (next_state, next_timer, should_eat) = advance_catch_or_eat(state, timer);
            if should_eat {
                ate_on_tick = Some(tick);
            }
            state = next_state;
            timer = next_timer;
            if state == TongueState::Done {
                assert_eq!(tick, 11, "tongue animation should span 11 ticks total");
                break;
            }
        }

        assert_eq!(ate_on_tick, Some(7), "vanilla eats on the 7th catch tick");
    }

    #[test]
    fn non_animation_states_pass_through_unchanged() {
        let (state, timer, should_eat) = advance_catch_or_eat(TongueState::MoveToTarget, 3);
        assert_eq!(state, TongueState::MoveToTarget);
        assert_eq!(timer, 3);
        assert!(!should_eat);
    }
}
