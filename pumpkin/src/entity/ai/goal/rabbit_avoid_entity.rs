use std::sync::Weak;

use super::avoid_entity::AvoidEntityGoal;
use super::{Controls, Goal, GoalFuture};
use crate::entity::mob::Mob;
use crate::entity::passive::rabbit::{RabbitEntity, RabbitVariant};
use pumpkin_data::entity::{EntityType, MobCategory};

/// Vanilla `Rabbit.RabbitAvoidEntityGoal`: identical to the generic `AvoidEntityGoal` except
/// `canUse()` additionally requires the rabbit not be the killer-bunny (`EVIL`) variant.
pub struct RabbitAvoidEntityGoal {
    inner: AvoidEntityGoal,
    rabbit: Weak<RabbitEntity>,
}

impl RabbitAvoidEntityGoal {
    #[must_use]
    pub fn new(
        flee_type: &'static EntityType,
        flee_distance: f64,
        slow_speed: f64,
        fast_speed: f64,
        rabbit: Weak<RabbitEntity>,
    ) -> Box<Self> {
        Box::new(Self {
            inner: AvoidEntityGoal::new(flee_type, flee_distance, slow_speed, fast_speed),
            rabbit,
        })
    }

    /// Vanilla `new Rabbit.RabbitAvoidEntityGoal<>(this, Monster.class, 4.0F, 2.2, 2.2)`
    /// (`Rabbit.registerGoals`, line 103).
    #[must_use]
    pub fn new_for_category(
        category: &'static MobCategory,
        flee_distance: f64,
        slow_speed: f64,
        fast_speed: f64,
        rabbit: Weak<RabbitEntity>,
    ) -> Box<Self> {
        Box::new(Self {
            inner: AvoidEntityGoal::new_for_category(
                category,
                flee_distance,
                slow_speed,
                fast_speed,
            ),
            rabbit,
        })
    }
}

impl Goal for RabbitAvoidEntityGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let is_evil = self
                .rabbit
                .upgrade()
                .is_some_and(|r| r.get_variant() == RabbitVariant::Evil);
            if is_evil {
                return false;
            }
            self.inner.can_start(mob).await
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        self.inner.should_continue(mob)
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        self.inner.start(mob)
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        self.inner.stop(mob)
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        self.inner.tick(mob)
    }

    fn should_run_every_tick(&self) -> bool {
        self.inner.should_run_every_tick()
    }

    fn controls(&self) -> Controls {
        self.inner.controls()
    }
}
