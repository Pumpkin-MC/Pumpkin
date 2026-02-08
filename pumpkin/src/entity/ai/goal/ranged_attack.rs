use super::{Controls, Goal};
use crate::entity::ai::goal::GoalFuture;
use crate::entity::ai::path::NavigatorGoal;
use crate::entity::mob::Mob;

/// A goal that makes the mob attack its target from range.
///
/// The mob navigates toward its target when out of range, then stands and
/// fires at a fixed interval. Projectile spawning is not yet implemented —
/// the goal currently deals direct damage as a placeholder.
///
/// Used by: Skeleton, Blaze, Ghast, Pillager, Drowned, Witch.
pub struct RangedAttackGoal {
    goal_control: Controls,
    speed: f64,
    attack_interval: i32,
    attack_radius_sq: f64,
    cooldown: i32,
    seen_target_ticks: i32,
}

impl RangedAttackGoal {
    #[must_use]
    pub fn new(speed: f64, attack_interval: i32, attack_radius: f64) -> Box<Self> {
        Box::new(Self {
            goal_control: Controls::MOVE | Controls::LOOK,
            speed,
            attack_interval,
            attack_radius_sq: attack_radius * attack_radius,
            cooldown: 0,
            seen_target_ticks: 0,
        })
    }
}

impl Goal for RangedAttackGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let target = mob.get_mob_entity().target.lock().await;
            let Some(target) = target.as_ref() else {
                return false;
            };
            target.get_entity().is_alive()
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let target = mob.get_mob_entity().target.lock().await;
            let Some(target) = target.as_ref() else {
                return false;
            };
            target.get_entity().is_alive()
        })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.cooldown = 0;
            self.seen_target_ticks = 0;
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.seen_target_ticks = 0;
            self.cooldown = 0;
            let mut navigator = mob.get_mob_entity().navigator.lock().await;
            navigator.cancel();
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            let target = mob.get_mob_entity().target.lock().await;
            let Some(target) = target.as_ref() else {
                return;
            };

            let mob_pos = mob.get_entity().pos.load();
            let target_pos = target.get_entity().pos.load();
            let dist_sq = mob_pos.squared_distance_to_vec(&target_pos);

            // Look at target
            mob.get_entity().look_at(target_pos);

            if dist_sq <= self.attack_radius_sq {
                // In range — stop moving, increment seen ticks
                self.seen_target_ticks += 1;
                let mut navigator = mob.get_mob_entity().navigator.lock().await;
                navigator.cancel();
            } else {
                // Out of range — move closer
                self.seen_target_ticks = 0;
                let mut navigator = mob.get_mob_entity().navigator.lock().await;
                navigator.set_progress(NavigatorGoal {
                    current_progress: mob_pos,
                    destination: target_pos,
                    speed: self.speed,
                });
            }

            // Attack cooldown
            self.cooldown -= 1;
            if self.cooldown <= 0 && self.seen_target_ticks > 0 {
                self.cooldown = self.attack_interval;

                // TODO: Spawn actual projectile (arrow, fireball, trident, potion)
                // based on the mob type. For now this is a stub — the ranged attack
                // goal provides correct targeting and timing behavior, but projectile
                // creation requires the projectile entity system.
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
