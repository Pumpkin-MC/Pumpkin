use super::{Controls, Goal};
use crate::entity::ai::goal::melee_attack_goal::MeleeAttackGoal;
use crate::entity::mob::Mob;
use async_trait::async_trait;

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

#[async_trait]
impl Goal for ZombieAttackGoal {
    async fn can_start(&mut self, mob: &dyn Mob) -> bool {
        self.melee_attack_goal.can_start(mob).await
    }

    async fn should_continue(&self, mob: &dyn Mob) -> bool {
        self.melee_attack_goal.should_continue(mob).await
    }

    async fn start(&mut self, mob: &dyn Mob) {
        self.melee_attack_goal.start(mob).await;
        self.ticks = 0;
    }

    async fn stop(&mut self, mob: &dyn Mob) {
        self.melee_attack_goal.stop(mob).await;
        mob.get_mob_entity().set_attacking(false);
    }

    async fn tick(&mut self, mob: &dyn Mob) {
        self.melee_attack_goal.tick(mob).await;
        self.ticks += 1;
        if self.ticks >= 5
            && self.melee_attack_goal.cooldown < self.melee_attack_goal.get_max_cooldown()
        {
            mob.get_mob_entity().set_attacking(true);
        } else {
            mob.get_mob_entity().set_attacking(false);
        }
    }

    fn should_run_every_tick(&self) -> bool {
        self.melee_attack_goal.should_run_every_tick()
    }

    fn controls(&self) -> Controls {
        self.melee_attack_goal.controls()
    }
}
