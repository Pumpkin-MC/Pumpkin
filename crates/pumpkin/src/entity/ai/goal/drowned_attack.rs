use std::sync::atomic::Ordering::Relaxed;

use super::drowned_util::is_bright_outside;
use super::zombie_attack::ZombieAttackGoal;
use super::{Controls, Goal, GoalFuture};
use crate::entity::mob::Mob;

/// `Drowned.DrownedAttackGoal` (`Drowned.java:323-340`): the same melee behavior as
/// `ZombieAttackGoal`, additionally gated on `Drowned#okTarget` every tick it's asked whether
/// to (continue to) run.
pub struct DrownedAttackGoal {
    melee: Box<ZombieAttackGoal>,
}

impl DrownedAttackGoal {
    #[must_use]
    pub fn new(speed: f64, pause_when_mob_idle: bool) -> Box<Self> {
        Box::new(Self {
            melee: ZombieAttackGoal::new(speed, pause_when_mob_idle),
        })
    }

    /// `Drowned#okTarget`: `target != null && (!level.isBrightOutside() || target.isInWater())`.
    async fn ok_target(mob: &dyn Mob) -> bool {
        let Some(target) = mob.get_mob_entity().target.lock().await.clone() else {
            return false;
        };
        let world = mob.get_entity().world.load();
        !is_bright_outside(&world) || target.get_entity().touching_water.load(Relaxed)
    }
}

impl Goal for DrownedAttackGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { self.melee.can_start(mob).await && Self::ok_target(mob).await })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { self.melee.should_continue(mob).await && Self::ok_target(mob).await })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move { self.melee.start(mob).await })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move { self.melee.stop(mob).await })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move { self.melee.tick(mob).await })
    }

    fn should_run_every_tick(&self) -> bool {
        self.melee.should_run_every_tick()
    }

    fn controls(&self) -> Controls {
        self.melee.controls()
    }
}
