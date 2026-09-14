use crate::entity::ai::goal::{Controls, Goal};
use crate::entity::mob::Mob;
use rand::RngExt;

pub struct AmbientStandGoal {
    goal_control: Controls,
    cooldown: i32,
}

impl AmbientStandGoal {
    const fn reset_cooldown(&mut self) {
        // TODO: should be: this.cooldown = -entity.getMinAmbientStandDelay();
        // TODO: implement when Horses are implemented
        self.cooldown = 0;
    }
}

impl Goal for AmbientStandGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        self.cooldown += 1;
        if self.cooldown > 0 && mob.get_random().random_range(0..1000) < self.cooldown {
            self.reset_cooldown();
        }

        false
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
