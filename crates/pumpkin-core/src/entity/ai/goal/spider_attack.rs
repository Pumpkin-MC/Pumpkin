use rand::RngExt;

use super::{Controls, Goal};
use crate::entity::ai::goal::melee_attack::MeleeAttackGoal;
use crate::entity::mob::Mob;

/// Melee that gives up in daylight: a 1 in 100 chance per check to drop the target.
pub struct SpiderAttackGoal {
    melee_attack_goal: MeleeAttackGoal,
}

impl SpiderAttackGoal {
    #[must_use]
    pub fn new(speed: f64, pause_when_mob_idle: bool) -> Box<Self> {
        Box::new(Self {
            melee_attack_goal: MeleeAttackGoal::new(speed, pause_when_mob_idle),
        })
    }
}

impl Goal for SpiderAttackGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        self.melee_attack_goal.can_start(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        // Roll first: the daylight lookup only runs on the rare hit.
        if mob.get_random().random_range(0..100) == 0 && mob.get_mob_entity().is_in_daylight() {
            mob.set_mob_target(None);
            return false;
        }
        self.melee_attack_goal.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.melee_attack_goal.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.melee_attack_goal.stop(mob);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.melee_attack_goal.tick(mob);
    }

    fn should_run_every_tick(&self) -> bool {
        self.melee_attack_goal.should_run_every_tick()
    }

    fn controls(&self) -> Controls {
        self.melee_attack_goal.controls()
    }
}
