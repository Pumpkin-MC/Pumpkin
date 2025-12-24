use super::{Controls, Goal};
use crate::entity::{ai::goal::GoalFuture, mob::Mob};
use rand::Rng;

#[allow(dead_code)]
pub struct AmbientStandGoal {
    goal_control: Controls,
    cooldown: i32,
}

impl AmbientStandGoal {
    fn reset_cooldown(&mut self) {
        // TODO: should be: this.cooldown = -entity.getMinAmbientStandDelay();
        // TODO: implement when Horses are implemented
        self.cooldown = 0;
    }
}

impl Goal for AmbientStandGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            self.cooldown += 1;
            if self.cooldown > 0 && mob.get_random().random_range(0..1000) < self.cooldown {
                self.reset_cooldown();
            }

            false
        })
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
