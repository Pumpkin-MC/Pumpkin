use std::sync::atomic::Ordering::Relaxed;

use super::{Controls, Goal, GoalFuture};
use crate::entity::mob::Mob;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::potion::Effect;
use rand::RngExt;

/// Vanilla `Axolotl.TOTAL_PLAYDEAD_TIME`.
const TOTAL_PLAYDEAD_TICKS: i32 = 200;

/// Vanilla `Axolotl.hurtServer`'s randomized part of the trigger condition:
/// `random.nextInt(3) == 0 && (random.nextInt(3) < damage || currentHealth / maxHealth < 0.5F)`.
/// Split out from `can_start` so the roll/threshold logic is unit-testable without a `Mob`.
const fn should_trigger_from_rolls(
    first_roll: i32,
    second_roll: i32,
    damage: f32,
    health: f32,
    max_health: f32,
) -> bool {
    if first_roll != 0 {
        return false;
    }
    let low_health = health / max_health < 0.5;
    low_health || (second_roll as f32) < damage
}

/// Makes an axolotl "play dead" for 200 ticks after taking non-lethal damage from an attacker
/// while in water.
///
/// Ports vanilla's `PLAY_DEAD_TICKS` memory plus the `Axolotl.hurtServer` trigger check and the
/// `PlayDead`/`ValidatePlayDead` brain behaviors. Vanilla source:
/// `net/minecraft/world/entity/animal/axolotl/{Axolotl,PlayDead,ValidatePlayDead}.java`.
///
/// Simplifications vs. vanilla:
/// - No brain/memory framework: the trigger condition is re-evaluated in `can_start` from
///   `LivingEntity::last_attacked_time`/`last_damage_taken` instead of a dedicated
///   `HURT_BY_ENTITY` memory, but checks the same underlying facts (was hit by an entity this
///   tick, while in water, non-lethally).
/// - Vanilla's `currentHealth` in the trigger check is health *before* the hit; this uses the
///   post-hit health, relying on "entity still alive" as the equivalent of vanilla's
///   `damage < currentHealth` (non-lethal) condition.
/// - Movement/look freezing is done by this goal claiming `Controls::MOVE | Controls::LOOK` at
///   the highest priority (so lower-priority goals can't run), rather than vanilla's
///   `AxolotlMoveControl`/`AxolotlLookControl` early-returning on `isPlayingDead()`.
/// - `WALK_TARGET`/`LOOK_TARGET` memory erasure on start has no equivalent since Pumpkin's goal
///   system has no memory to erase; claiming the controls achieves the same freeze.
pub struct AxolotlPlayDeadGoal {
    goal_control: Controls,
    ticks_remaining: i32,
}

impl AxolotlPlayDeadGoal {
    #[must_use]
    pub fn new() -> Self {
        Self {
            goal_control: Controls::MOVE | Controls::LOOK,
            ticks_remaining: 0,
        }
    }
}

impl Default for AxolotlPlayDeadGoal {
    fn default() -> Self {
        Self::new()
    }
}

impl Goal for AxolotlPlayDeadGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if self.ticks_remaining > 0 {
                return false;
            }

            let mob_entity = mob.get_mob_entity();
            if mob_entity.is_no_ai() {
                return false;
            }
            let living = &mob_entity.living_entity;

            // Vanilla: `source.getEntity() != null || source.getDirectEntity() != null` -- only
            // entities update `last_attacked_time`/`last_attacker_id` (environmental damage
            // doesn't), so "attacked this tick" implies an attacker entity.
            let age = living.entity.age.load(Relaxed);
            if living.last_attacked_time.load(Relaxed) != age {
                return false;
            }

            if !living.is_in_water() {
                return false;
            }

            let health = living.health.load();
            if health <= 0.0 {
                return false;
            }

            let mut rng = mob.get_random();
            let last_damage = living.last_damage_taken.load();
            should_trigger_from_rolls(
                rng.random_range(0..3),
                rng.random_range(0..3),
                last_damage,
                health,
                living.get_max_health(),
            )
        })
    }

    fn should_continue<'a>(&'a self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { self.ticks_remaining > 0 })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.ticks_remaining = TOTAL_PLAYDEAD_TICKS;

            let mob_entity = mob.get_mob_entity();
            mob_entity.navigator.lock().unwrap().stop();
            mob_entity
                .living_entity
                .not_targetable_as_enemy
                .store(true, Relaxed);

            // Vanilla `PlayDead.start`: `addEffect(new MobEffectInstance(REGENERATION, 200, 0))`.
            mob_entity
                .living_entity
                .add_effect(Effect {
                    effect_type: &StatusEffect::REGENERATION,
                    duration: TOTAL_PLAYDEAD_TICKS,
                    amplifier: 0,
                    ambient: false,
                    show_particles: true,
                    show_icon: true,
                    blend: false,
                })
                .await;
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.ticks_remaining = 0;
            mob.get_mob_entity()
                .living_entity
                .not_targetable_as_enemy
                .store(false, Relaxed);
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.ticks_remaining = (self.ticks_remaining - 1).max(0);
            // Keep the axolotl from drifting or being pathed away while playing dead.
            mob.get_mob_entity().navigator.lock().unwrap().stop();
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

#[cfg(test)]
mod tests {
    use super::should_trigger_from_rolls;

    #[test]
    fn first_roll_gates_everything() {
        // first_roll != 0 short-circuits regardless of how favorable the rest is.
        assert!(!should_trigger_from_rolls(1, 0, 10.0, 1.0, 14.0));
        assert!(!should_trigger_from_rolls(2, 2, 10.0, 14.0, 14.0));
    }

    #[test]
    fn low_health_triggers_without_needing_damage_roll() {
        // health/maxHealth < 0.5, second_roll (2) is not < damage (0.0), but low health still wins.
        assert!(should_trigger_from_rolls(0, 2, 0.0, 6.0, 14.0));
    }

    #[test]
    fn high_damage_triggers_even_at_full_health() {
        // health/maxHealth == 1.0 (not low), but second_roll (0) < damage (3.0).
        assert!(should_trigger_from_rolls(0, 0, 3.0, 14.0, 14.0));
    }

    #[test]
    fn healthy_and_low_damage_does_not_trigger() {
        assert!(!should_trigger_from_rolls(0, 2, 1.0, 14.0, 14.0));
    }
}
