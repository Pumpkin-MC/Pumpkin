use std::sync::atomic::Ordering;
use std::sync::{Arc, Weak};

use super::{Controls, Goal};
use crate::entity::ai::goal::GoalFuture;
use crate::entity::mob::Mob;
use crate::entity::mob::creeper::CreeperEntity;

pub struct CreeperIgniteGoal {
    goal_control: Controls,
    creeper: Weak<CreeperEntity>,
}

impl CreeperIgniteGoal {
    #[must_use]
    pub fn new(creeper: &Arc<CreeperEntity>) -> Self {
        Self {
            goal_control: Controls::MOVE,
            creeper: Arc::downgrade(creeper),
        }
    }
}

impl Goal for CreeperIgniteGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(creeper) = self.creeper.upgrade() else {
                return false;
            };

            let mob_entity = mob.get_mob_entity();
            let target_lock = mob_entity.target.lock().await;

            if creeper.fuse_speed.load(Ordering::Relaxed) > 0 {
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
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(creeper) = self.creeper.upgrade() {
                creeper.set_fuse_speed(-1).await;
            }
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(creeper) = self.creeper.upgrade() else {
                return;
            };

            let target_lock = mob.get_mob_entity().target.lock().await;

            let Some(target) = target_lock.as_ref() else {
                creeper.set_fuse_speed(-1).await;
                return;
            };

            let dist_sq = mob
                .get_entity()
                .pos
                .load()
                .squared_distance_to_vec(&target.get_entity().pos.load());

            if dist_sq > 49.0 {
                creeper.set_fuse_speed(-1).await;
            }
            // TODO: line of sight check (needs world raycast)
            else {
                creeper.set_fuse_speed(1).await;
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
