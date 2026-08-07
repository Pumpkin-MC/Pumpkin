use super::melee_attack::MeleeAttackGoal;
use super::{Controls, Goal, GoalFuture};
use crate::entity::mob::Mob;
use crate::entity::passive::polar_bear::PolarBearEntity;

/// `PolarBear.PolarBearMeleeAttackGoal` (PolarBear.java:304-336): on top of the generic melee
/// attack, a bear that's close to its target but not yet swinging rears up (`setStanding(true)`).
///
/// It also plays a warning growl once `ticksUntilNextAttack <= 10`. The generic
/// `MeleeAttackGoal` has no `checkAndPerformAttack` hook to override (see
/// `FoxMeleeAttackGoal`'s documented simplification for the same limitation), so this composes
/// with it instead of reimplementing its movement/pathing: it runs the inner goal unchanged for
/// movement and the actual attack, then separately derives the standing/warning state from the
/// inner goal's public `cooldown` field (vanilla's `ticksUntilNextAttack`) and target distance.
pub struct PolarBearMeleeAttackGoal {
    inner: MeleeAttackGoal,
}

impl PolarBearMeleeAttackGoal {
    #[must_use]
    pub fn new() -> Box<Self> {
        Box::new(Self {
            inner: MeleeAttackGoal::new(1.25, true),
        })
    }
}

impl Goal for PolarBearMeleeAttackGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        self.inner.can_start(mob)
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        self.inner.should_continue(mob)
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        self.inner.start(mob)
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.inner.stop(mob).await;
            if let Some(bear) = mob.cast_any().downcast_ref::<PolarBearEntity>() {
                bear.set_standing(false);
            }
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let old_cooldown = self.inner.cooldown;
            self.inner.tick(mob).await;

            let Some(bear) = mob.cast_any().downcast_ref::<PolarBearEntity>() else {
                return;
            };

            // The inner goal resets `cooldown` back up to its max only when it actually landed
            // an attack this tick -- vanilla's `canPerformAttack` branch, which also drops
            // standing.
            if self.inner.cooldown > old_cooldown {
                bear.set_standing(false);
                return;
            }

            let target = mob.get_mob_entity().target.lock().await.clone();
            let Some(target) = target else {
                bear.set_standing(false);
                return;
            };
            let target_entity = target.get_entity();

            let dist_sq = mob
                .get_entity()
                .pos
                .load()
                .squared_distance_to_vec(&target_entity.pos.load());
            let near_reach = f64::from(target_entity.entity_dimension.load().width) + 3.0;

            if dist_sq < near_reach * near_reach {
                if self.inner.cooldown <= 10 {
                    bear.set_standing(true);
                    bear.play_warning_sound();
                }
            } else {
                bear.set_standing(false);
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.inner.controls()
    }
}
