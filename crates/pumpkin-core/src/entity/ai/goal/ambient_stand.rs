use crate::entity::ai::goal::{Controls, Goal};
use crate::entity::mob::Mob;
use rand::RngExt;

pub struct AmbientStandGoal {
    goal_control: Controls,
    cooldown: i32,
}

impl Default for AmbientStandGoal {
    fn default() -> Self {
        Self {
            goal_control: Controls::MOVE | Controls::JUMP,
            cooldown: -120,
        }
    }
}

impl AmbientStandGoal {
    #[must_use]
    pub fn new() -> Self {
        Self {
            goal_control: Controls::MOVE | Controls::JUMP,
            cooldown: -120,
        }
    }

    const fn reset_cooldown(&mut self) {
        // Vanilla: -getAmbientStandInterval()
        self.cooldown = -120;
    }
}

impl Goal for AmbientStandGoal {
    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        false
    }

    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        self.cooldown += 1;
        if self.cooldown > 0 && mob.get_random().random_range(0..1000) < self.cooldown {
            self.reset_cooldown();
        }

        false
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
