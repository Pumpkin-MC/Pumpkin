use super::{Controls, Goal};
use crate::entity::ai::goal::GoalFuture;
use crate::entity::ai::goal::melee_attack_goal::MeleeAttackGoal;
use crate::entity::mob::Mob;

pub struct ZombieAttackGoal {
    melee_attack_goal: MeleeAttackGoal,
    ticks: i32,
}

impl ZombieAttackGoal {
    #[must_use]
    pub fn new(speed: f64, pause_when_mob_idle: bool) -> Box<Self> {
        Box::new(Self {
            melee_attack_goal: MeleeAttackGoal::new(speed, pause_when_mob_idle),
            ticks: 0,
        })
    }
}

impl Goal for ZombieAttackGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async { self.melee_attack_goal.can_start(mob).await })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async { self.melee_attack_goal.should_continue(mob).await })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.melee_attack_goal.start(mob).await;
            self.ticks = 0;
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.melee_attack_goal.stop(mob).await;
            // Assuming set_attacking is synchronous
            mob.get_mob_entity().set_attacking(false);
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.melee_attack_goal.tick(mob).await;
            self.ticks += 1;

            // Note: Accessing cooldown/get_max_cooldown is assumed synchronous.
            if self.ticks >= 5
                && self.melee_attack_goal.cooldown < self.melee_attack_goal.get_max_cooldown()
            {
                mob.get_mob_entity().set_attacking(true);
            } else {
                mob.get_mob_entity().set_attacking(false);
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        self.melee_attack_goal.should_run_every_tick()
    }

    fn controls(&self) -> Controls {
        self.melee_attack_goal.controls()
    }
}
