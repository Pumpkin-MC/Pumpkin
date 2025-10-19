use super::{Controls, Goal};
use crate::entity::mob::Mob;
use async_trait::async_trait;
use rand::Rng;

#[allow(dead_code)]
#[derive(Default)]
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

#[async_trait]
impl Goal for AmbientStandGoal {
    async fn can_start(&mut self, mob: &dyn Mob) -> bool {
        self.cooldown += 1;
        if self.cooldown > 0 && mob.get_random().random_range(0..1000) < self.cooldown {
            self.reset_cooldown();
        }

        false
    }
    async fn should_continue(&self, _mob: &dyn Mob) -> bool {
        false
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
