//! Schedule movement for villagers.
//!
//! Walk home at dusk and to the job site in the morning — a stand-in for the
//! vanilla brain schedule (SleepInBed / WorkAtPoi walk targets); the sleeping
//! and job logic itself lives in the villager's `mob_tick`.

use pumpkin_util::math::position::BlockPos;

use super::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::entity::passive::villager::VillagerEntity;

/// Close enough for `mob_tick` to lie down (2 blocks squared).
const HOME_REACH_SQ: f64 = 4.0;
const WORK_REACH_SQ: f64 = 6.25;

pub struct VillagerScheduleGoal {
    speed: f64,
    target: Option<BlockPos>,
    repath_delay: i32,
}

impl VillagerScheduleGoal {
    #[must_use]
    pub const fn new(speed: f64) -> Self {
        Self {
            speed,
            target: None,
            repath_delay: 0,
        }
    }

    /// Returns the scheduled destination for the current time of day.
    async fn schedule_target(mob: &dyn Mob) -> Option<BlockPos> {
        let villager = mob.cast_any().downcast_ref::<VillagerEntity>()?;
        let entity = mob.get_entity();
        let world = entity.world.load();
        let time = world.level_time.lock().await.time_of_day.rem_euclid(24000);

        // Vanilla schedule: work 2000-9000, sleep from 12000.
        let (target, reach_sq) = if (12000..=23000).contains(&time) {
            (villager.get_home(), HOME_REACH_SQ)
        } else if (2000..9000).contains(&time) {
            (villager.get_job_site(), WORK_REACH_SQ)
        } else {
            (None, 0.0)
        };
        let target = target?;
        let distance_sq = entity
            .pos
            .load()
            .squared_distance_to_vec(&target.to_centered_f64());
        (distance_sq > reach_sq).then_some(target)
    }
}

impl Goal for VillagerScheduleGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            self.target = Self::schedule_target(mob).await;
            self.target.is_some()
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if self.target.is_none() {
                return false;
            }
            Self::schedule_target(mob).await == self.target
        })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.repath_delay = 0;
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.target = None;
            mob.get_mob_entity().navigator.lock().unwrap().stop();
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.repath_delay -= 1;
            if self.repath_delay > 0 {
                return;
            }
            self.repath_delay = to_goal_ticks(20);
            let Some(target) = self.target else {
                return;
            };
            let mob_pos = mob.get_entity().pos.load();
            let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
            navigator.set_progress(NavigatorGoal::new(
                mob_pos,
                target.to_centered_f64(),
                self.speed,
            ));
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}
