use std::sync::Arc;
use std::sync::atomic::Ordering;

use super::{Controls, Goal};
use crate::entity::ai::goal::GoalFuture;
use crate::entity::mob::Mob;
use crate::entity::mob::creeper::CreeperEntity;

pub struct CreeperIgniteGoal {
    goal_control: Controls,
    creeper: Arc<CreeperEntity>,
}

impl CreeperIgniteGoal {
    #[must_use]
    pub const fn new(creeper: Arc<CreeperEntity>) -> Self {
        Self {
            goal_control: Controls::MOVE,
            creeper,
        }
    }
}

impl Goal for CreeperIgniteGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let creeper = mob.get_mob_entity();
            let target_lock = creeper.target.lock().await;

            if self.creeper.fuse_speed.load(Ordering::Relaxed) > 0 {
                return true;
            }

            if let Some(target) = target_lock.as_ref() {
                let dist_sq = mob
                    .get_entity()
                    .pos
                    .load()
                    .squared_distance_to_vec(&target.get_entity().pos.load());
                return dist_sq < 9.0;
            }

            false
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let mut navigator = mob.get_mob_entity().navigator.lock().await;
            navigator.stop();

            // TODO
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            // TODO
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let target_lock = mob.get_mob_entity().target.lock().await;

            let Some(target) = target_lock.as_ref() else {
                self.creeper.set_fuse_speed(-1);
                return;
            };

            let dist_sq = mob
                .get_entity()
                .pos
                .load()
                .squared_distance_to_vec(&target.get_entity().pos.load());

            if dist_sq > 49.0 {
                self.creeper.set_fuse_speed(-1);
            }
            // TODO: Check line of sight
            else {
                self.creeper.set_fuse_speed(1);
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
