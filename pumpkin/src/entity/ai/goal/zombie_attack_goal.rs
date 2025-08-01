use super::{Goal, GoalControl};
use crate::entity::ai::goal::melee_attack_goal::MeleeAttackGoal;
use crate::entity::mob::Mob;
use async_trait::async_trait;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering::Relaxed;

pub struct ZombieAttackGoal {
    melee_attack_goal: MeleeAttackGoal,
    ticks: AtomicI32,
}

impl ZombieAttackGoal {
    #[must_use]
    pub fn new(speed: f64, pause_when_mob_idle: bool) -> Self {
        Self {
            melee_attack_goal: MeleeAttackGoal::new(speed, pause_when_mob_idle),
            ticks: AtomicI32::new(0),
        }
    }
}

#[async_trait]
impl Goal for ZombieAttackGoal {
    async fn can_start(&self, mob: &dyn Mob) -> bool {
        self.melee_attack_goal.can_start(mob).await
    }

    async fn should_continue(&self, mob: &dyn Mob) -> bool {
        self.melee_attack_goal.should_continue(mob).await
    }

    async fn start(&self, mob: &dyn Mob) {
        self.melee_attack_goal.start(mob).await;
        self.ticks.store(0, Relaxed);
    }

    async fn stop(&self, mob: &dyn Mob) {
        self.melee_attack_goal.stop(mob).await;
        mob.get_mob_entity().set_attacking(false);
    }

    async fn tick(&self, mob: &dyn Mob) {
        self.melee_attack_goal.tick(mob).await;
        let ticks = self.ticks.fetch_add(1, Relaxed) + 1;
        if ticks >= 5
            && self.melee_attack_goal.cooldown.load(Relaxed)
                < self.melee_attack_goal.get_max_cooldown()
        {
            mob.get_mob_entity().set_attacking(true);
        } else {
            mob.get_mob_entity().set_attacking(false);
        }
    }

    fn should_run_every_tick(&self) -> bool {
        self.melee_attack_goal.should_run_every_tick()
    }

    fn get_goal_control(&self) -> &GoalControl {
        self.melee_attack_goal.get_goal_control()
    }
}
