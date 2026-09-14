use std::sync::Arc;
use std::sync::atomic::Ordering;

use super::{Controls, Goal};

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
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let creeper = mob.get_mob_entity();
        let target_lock = creeper
            .target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

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
    }

    fn start(&mut self, mob: &dyn Mob) {
        let mut navigator = mob
            .get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        navigator.stop();
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.creeper.set_fuse_speed(-1);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let target_lock = mob.get_mob_entity().get_target();

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
        // TODO: line of sight check (needs world raycast)
        else {
            self.creeper.set_fuse_speed(1);
        }
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
