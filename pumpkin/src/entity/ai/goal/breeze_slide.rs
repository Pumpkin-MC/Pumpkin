//! Port of `Slide.java`.
//!
//! The lowest-priority fallback in the FIGHT activity, used to reposition around the
//! target when the breeze can neither shoot (out of range) nor jump (on cooldown or too
//! close). Vanilla drives this purely through a `WALK_TARGET` memory consumed by a
//! generic movement behavior; Pumpkin has no such indirection, so this goal drives the
//! `Navigator` directly instead.

use std::sync::Weak;
use std::sync::atomic::Ordering::Relaxed;

use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::xoroshiro128::Xoroshiro;
use pumpkin_util::random::{RandomGenerator, RandomImpl, get_seed};

use crate::entity::{
    ai::goal::{Controls, Goal, GoalFuture, breeze_util::random_point_behind_target},
    ai::pathfinder::NavigatorGoal,
    mob::{Mob, breeze::BreezeEntity},
};

// BreezeAi.java: JUMP_CIRCLE_INNER_RADIUS = 4.0F, JUMP_CIRCLE_DISTANCE_Y = 10.
const INNER_CIRCLE_XZ: f64 = 4.0;
const INNER_CIRCLE_Y: f64 = 10.0;
const SLIDE_SPEED: f64 = 0.6; // BreezeAi.SPEED_MULTIPLIER_WHEN_SLIDING
// Fight-activity `Slide` has no explicit duration in the ported source; this reuses
// the IDLE-activity `SlideToTargetSink` timeout band (20-40 ticks) as a reasonable cap
// so the goal can't get stuck if the navigator never reports arrival.
const MAX_DURATION_TICKS: i32 = 40;

pub struct BreezeSlideGoal {
    breeze: Weak<BreezeEntity>,
    elapsed_ticks: i32,
}

impl BreezeSlideGoal {
    #[must_use]
    pub const fn new(breeze: Weak<BreezeEntity>) -> Self {
        Self {
            breeze,
            elapsed_ticks: 0,
        }
    }

    /// `Breeze.withinInnerCircleRange`: `closerThan(pos, 4.0, 10.0)` - horizontal 4,
    /// vertical 10, not a spherical radius.
    fn within_inner_circle_range(breeze_pos: Vector3<f64>, target_pos: Vector3<f64>) -> bool {
        let d = target_pos.sub(&breeze_pos);
        d.horizontal_length_squared() < INNER_CIRCLE_XZ * INNER_CIRCLE_XZ
            && d.y.abs() < INNER_CIRCLE_Y
    }

    /// `Slide.randomPointInMiddleCircle`.
    fn random_point_in_middle_circle(
        breeze_pos: Vector3<f64>,
        target_pos: Vector3<f64>,
        random: &mut RandomGenerator,
    ) -> Vector3<f64> {
        let direction = target_pos.sub(&breeze_pos);
        let distance = direction.length() - (8.0 - f64::from(random.next_f32()) * 4.0);
        breeze_pos.add(&direction.normalize().multiply(distance, distance, distance))
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

    /// Simplified stand-in for vanilla's `DefaultRandomPos.getPosAway`, which searches
    /// the pathfinder's node cache for a walkable point away from a position. Pumpkin
    /// has no equivalent node cache to query, so this samples a point directly away
    /// from the target at the same distance band (5-10 blocks) and relies on the
    /// distance/line-of-sight checks below to reject bad picks, falling back to the
    /// same points `LongJump` uses when it doesn't pan out.
    fn pos_away_from(
        breeze_pos: Vector3<f64>,
        target_pos: Vector3<f64>,
        random: &mut RandomGenerator,
    ) -> Vector3<f64> {
        let away = breeze_pos.sub(&target_pos).normalize();
        let distance = 5.0 + f64::from(random.next_f32()) * 5.0;
        breeze_pos.add(&away.multiply(distance, distance, distance))
    }
}

impl Goal for BreezeSlideGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(breeze) = self.breeze.upgrade() else {
                return false;
            };
            if breeze.shoot_window_ticks() > 0 || breeze.jump_cooldown_ticks() > 0 {
                return false;
            }
            let entity = mob.get_entity();
            if !entity.on_ground.load(Relaxed) || entity.touching_water.load(Relaxed) {
                return false;
            }
            if !mob.get_mob_entity().navigator.lock().unwrap().is_idle() {
                return false;
            }

            let target = breeze.mob_entity.target.lock().await.clone();
            target.is_some_and(|t| t.get_entity().is_alive())
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            self.elapsed_ticks < MAX_DURATION_TICKS
                && !mob.get_mob_entity().navigator.lock().unwrap().is_idle()
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.elapsed_ticks = 0;
            let Some(breeze) = self.breeze.upgrade() else {
                return;
            };
            let Some(target) = breeze.mob_entity.target.lock().await.clone() else {
                return;
            };

            let breeze_pos = mob.get_entity().pos.load();
            let target_pos = target.get_entity().pos.load();
            let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()));

            let mut destination = None;
            if Self::within_inner_circle_range(breeze_pos, target_pos) {
                let candidate = Self::pos_away_from(breeze_pos, target_pos, &mut random);
                if Self::has_line_of_sight(mob, candidate).await
                    && target_pos.squared_distance_to_vec(&candidate)
                        > target_pos.squared_distance_to_vec(&breeze_pos)
                {
                    destination = Some(candidate);
                }
            }

            let destination = destination.unwrap_or_else(|| {
                if random.next_bool() {
                    random_point_behind_target(
                        target_pos,
                        target.get_entity().head_yaw.load(),
                        &mut random,
                    )
                } else {
                    Self::random_point_in_middle_circle(breeze_pos, target_pos, &mut random)
                }
            });

            mob.get_mob_entity()
                .navigator
                .lock()
                .unwrap()
                .set_progress(NavigatorGoal {
                    current_progress: breeze_pos,
                    destination,
                    speed: SLIDE_SPEED,
                });
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            mob.get_mob_entity().navigator.lock().unwrap().stop();
        })
    }

    fn tick<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.elapsed_ticks += 1;
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}

#[cfg(test)]
mod tests {
    use super::BreezeSlideGoal;
    use pumpkin_util::math::vector3::Vector3;

    #[test]
    fn inner_circle_uses_horizontal_and_vertical_bounds_separately() {
        let breeze_pos = Vector3::new(0.0, 64.0, 0.0);
        // Within horizontal 4 blocks, well within vertical 10.
        assert!(BreezeSlideGoal::within_inner_circle_range(
            breeze_pos,
            Vector3::new(3.0, 64.0, 0.0)
        ));
        // Exactly at the horizontal boundary is NOT closer-than (strict <).
        assert!(!BreezeSlideGoal::within_inner_circle_range(
            breeze_pos,
            Vector3::new(4.0, 64.0, 0.0)
        ));
        // Close horizontally but far vertically (e.g. flying far overhead).
        assert!(!BreezeSlideGoal::within_inner_circle_range(
            breeze_pos,
            Vector3::new(1.0, 75.0, 0.0)
        ));
    }
}
