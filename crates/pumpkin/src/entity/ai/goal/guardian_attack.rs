use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::Difficulty;

use crate::entity::{
    EntityBase,
    ai::goal::{Controls, Goal, GoalFuture},
    mob::Mob,
};

/// Vanilla `Guardian.ATTACK_TIME` (`Guardian#getAttackDuration`), overridden by
/// `ElderGuardian#getAttackDuration`.
const ATTACK_TIME: i32 = 80;
const ELDER_ATTACK_TIME: i32 = 60;

/// `GuardianAttackGoal#start` seeds the counter below zero, so the real windup is
/// `attack_duration + 10` ticks.
const START_DELAY: i32 = -10;

/// `GuardianAttackSelector` / `canContinueToUse` both gate on a squared distance of 9.
const MIN_ATTACK_DISTANCE_SQ: f64 = 9.0;

/// Port of `Guardian.GuardianAttackGoal` (net.minecraft.world.entity.monster.Guardian).
pub struct GuardianAttackGoal {
    attack_time: i32,
}

impl Default for GuardianAttackGoal {
    fn default() -> Self {
        Self::new()
    }
}

impl GuardianAttackGoal {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            attack_time: START_DELAY,
        }
    }

    fn is_elder(mob: &dyn Mob) -> bool {
        mob.get_entity().entity_type == &EntityType::ELDER_GUARDIAN
    }

    const fn attack_duration(elder: bool) -> i32 {
        if elder {
            ELDER_ATTACK_TIME
        } else {
            ATTACK_TIME
        }
    }

    /// `GuardianAttackGoal#tick`: 1 base, +2 on Hard, +2 for elder guardians.
    fn magic_damage(difficulty: Difficulty, elder: bool) -> f32 {
        let mut damage = 1.0;
        if difficulty == Difficulty::Hard {
            damage += 2.0;
        }
        if elder {
            damage += 2.0;
        }
        damage
    }

    /// Vanilla `Guardian#setActiveAttackTarget`; the client renders the beam from this id.
    fn set_active_attack_target(mob: &dyn Mob, entity_id: i32) {
        mob.get_entity().send_meta_data(
            &[Metadata::new(
                TrackedData::ID_ATTACK_TARGET,
                MetaDataType::INT,
                VarInt(entity_id),
            )],
            None,
        );
    }

    async fn has_line_of_sight(mob: &dyn Mob, target: &dyn EntityBase) -> bool {
        mob.get_mob_entity().has_line_of_sight(target).await
    }

    fn distance_sq(mob: &dyn Mob, target: &dyn EntityBase) -> f64 {
        let a = mob.get_entity().pos.load();
        let b = target.get_entity().pos.load();
        let (dx, dy, dz) = (b.x - a.x, b.y - a.y, b.z - a.z);
        dx * dx + dy * dy + dz * dz
    }
}

impl Goal for GuardianAttackGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let target = mob.get_mob_entity().target.lock().await.clone();
            target.is_some_and(|target| target.get_entity().is_alive())
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let target = mob.get_mob_entity().target.lock().await.clone();
            let Some(target) = target else {
                return false;
            };
            if !target.get_entity().is_alive() {
                return false;
            }
            // Elder guardians keep firing at point-blank range; regular ones break off.
            Self::is_elder(mob) || Self::distance_sq(mob, target.as_ref()) > MIN_ATTACK_DISTANCE_SQ
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.attack_time = START_DELAY;
            mob.get_mob_entity().navigator.lock().unwrap().stop();

            let target = mob.get_mob_entity().target.lock().await.clone();
            if let Some(target) = target {
                mob.get_mob_entity()
                    .look_control
                    .lock()
                    .unwrap()
                    .look_at_entity_with_range(&target, 90.0, 90.0);
            }
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            Self::set_active_attack_target(mob, 0);
            *mob.get_mob_entity().target.lock().await = None;
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let mob_entity = mob.get_mob_entity();
            let target = mob_entity.target.lock().await.clone();
            let Some(target) = target else {
                return;
            };

            mob_entity.navigator.lock().unwrap().stop();
            mob_entity
                .look_control
                .lock()
                .unwrap()
                .look_at_entity_with_range(&target, 90.0, 90.0);

            if !Self::has_line_of_sight(mob, target.as_ref()).await {
                *mob_entity.target.lock().await = None;
                return;
            }

            let elder = Self::is_elder(mob);
            self.attack_time += 1;

            if self.attack_time == 0 {
                Self::set_active_attack_target(mob, target.get_entity().entity_id);
                mob.get_entity()
                    .world
                    .load()
                    .send_entity_status(mob.get_entity(), EntityStatus::GuardianAttackSound);
            } else if self.attack_time >= Self::attack_duration(elder) {
                let world = mob.get_entity().world.load();
                let damage = Self::magic_damage(world.level_info.load().difficulty, elder);
                let attacker = world.get_entity_by_id(mob.get_entity().entity_id);

                target
                    .damage_with_context(
                        target.as_ref(),
                        damage,
                        pumpkin_data::damage::DamageType::INDIRECT_MAGIC,
                        None,
                        attacker.as_deref(),
                        attacker.as_deref(),
                    )
                    .await;
                mob.try_attack(target.as_ref()).await;

                *mob_entity.target.lock().await = None;
            }
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}

#[cfg(test)]
mod tests {
    use super::GuardianAttackGoal;
    use pumpkin_util::Difficulty;

    #[test]
    fn magic_damage_matches_vanilla() {
        assert!(
            (GuardianAttackGoal::magic_damage(Difficulty::Normal, false) - 1.0).abs()
                < f32::EPSILON
        );
        assert!(
            (GuardianAttackGoal::magic_damage(Difficulty::Hard, false) - 3.0).abs() < f32::EPSILON
        );
        assert!(
            (GuardianAttackGoal::magic_damage(Difficulty::Normal, true) - 3.0).abs() < f32::EPSILON
        );
        assert!(
            (GuardianAttackGoal::magic_damage(Difficulty::Hard, true) - 5.0).abs() < f32::EPSILON
        );
    }

    #[test]
    fn attack_duration_matches_vanilla() {
        assert_eq!(GuardianAttackGoal::attack_duration(false), 80);
        assert_eq!(GuardianAttackGoal::attack_duration(true), 60);
    }
}
