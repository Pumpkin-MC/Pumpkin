use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;

use super::{Controls, Goal};
use crate::entity::EntityBase;
use crate::entity::ai::goal::GoalFuture;
use crate::entity::ai::goal::track_target::TrackTargetGoal;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::mob::Mob;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityType;
use pumpkin_data::tag::{self, Taggable};

/// Vanilla `Raider.class` membership check, approximated via the `#minecraft:raiders` tag
/// (Witch, Pillager, Vindicator, Evoker, Illusioner, Ravager, Ravager rider Pillager, etc.).
#[must_use]
pub fn is_raider(entity_type: &EntityType) -> bool {
    entity_type.has_tag(&tag::EntityType::MINECRAFT_RAIDERS)
}

pub struct RevengeGoal {
    track_target_goal: TrackTargetGoal,
    target: Option<Arc<dyn EntityBase>>,
    last_attacked_time: i32,
    target_predicate: TargetPredicate,
    /// Vanilla `HurtByTargetGoal(this, Raider.class)`'s `toIgnoreDamage`: an attacker of this
    /// class is ignored entirely, so raid-mates don't retaliate against friendly fire from
    /// other raiders. Used by Vex (`Vex.java:93`) and Witch (`Witch.java:72`).
    exclude_raiders: bool,
    /// Vanilla `PolarBear.PolarBearHurtByTargetGoal::alertOther` override: only alert nearby
    /// same-species mobs that aren't babies (`PolarBear.java:296-301`).
    alert_only_adults: bool,
    /// Vanilla `PolarBear.PolarBearHurtByTargetGoal` never calls `setAlertOthers()`, so the
    /// base class's unconditional `if (alertSameType) alertOthers()` never fires for it; only
    /// its `start()` override calls `alertOthers()` directly, and only when the hurt bear
    /// itself is a baby (`PolarBear.java:284-291`). When set, gates the alert loop below on
    /// that condition instead of always running it.
    alert_only_when_self_is_baby: bool,
}

impl RevengeGoal {
    #[must_use]
    pub fn new(check_visibility: bool) -> Self {
        let target_predicate = TargetPredicate::create_attackable()
            .ignore_visibility()
            .ignore_distance_scaling_factor();
        Self {
            track_target_goal: TrackTargetGoal::with_default(check_visibility),
            target: None,
            last_attacked_time: 0,
            target_predicate,
            exclude_raiders: false,
            alert_only_adults: false,
            alert_only_when_self_is_baby: false,
        }
    }

    #[must_use]
    pub const fn exclude_raiders(mut self) -> Self {
        self.exclude_raiders = true;
        self
    }

    #[must_use]
    pub const fn alert_only_adults(mut self) -> Self {
        self.alert_only_adults = true;
        self
    }

    #[must_use]
    pub const fn alert_only_when_self_is_baby(mut self) -> Self {
        self.alert_only_when_self_is_baby = true;
        self
    }
}

impl Goal for RevengeGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let mob_entity = mob.get_mob_entity();
            let living = &mob_entity.living_entity;

            let attacked_time = living.last_attacked_time.load(Relaxed);
            if attacked_time == self.last_attacked_time {
                return false;
            }

            let attacker_id = living.last_attacker_id.load(Relaxed);
            if attacker_id == 0 {
                return false;
            }

            let world = living.entity.world.load();
            let Some(attacker) = world.get_entity_by_id(attacker_id) else {
                return false;
            };

            let Some(attacker_living) = attacker.get_living_entity() else {
                return false;
            };

            if self.exclude_raiders && is_raider(attacker.get_entity().entity_type) {
                return false;
            }

            // Vanilla `TamableAnimal::canAttack` unconditionally excludes the mob's own owner
            // from any attack target, regardless of which targeting goal found them; for
            // non-tameable mobs `get_owner_uuid()` is always `None` so this is a no-op.
            if mob.get_owner_uuid() == Some(attacker.get_entity().entity_uuid) {
                return false;
            }

            // Vanilla `TargetingConditions.test`'s combat branch (`TargetingConditions.java:78`)
            // consults `targeter.canAttack(target)`; `HurtByTargetGoal` reaches it through
            // `TargetGoal.canAttack`. This is what stops a player-created iron golem from
            // retaliating against the player who punched it.
            if !mob.can_attack(attacker.get_entity()) {
                return false;
            }

            if TrackTargetGoal::is_allied(mob, attacker.as_ref()).await {
                return false;
            }

            if !self
                .target_predicate
                .test(&world, Some(&mob_entity.living_entity), attacker_living)
                .await
            {
                return false;
            }

            self.target = Some(attacker);
            true
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async { self.track_target_goal.should_continue(mob).await })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            mob.set_mob_target(self.target.clone()).await;

            let mob_entity = mob.get_mob_entity();
            self.last_attacked_time = mob_entity.living_entity.last_attacked_time.load(Relaxed);
            self.track_target_goal.max_time_without_visibility = 300;
            self.track_target_goal.start(mob).await;

            let Some(target) = self.target.as_ref() else {
                return;
            };
            if self.alert_only_when_self_is_baby && mob.get_entity().age.load(Relaxed) >= 0 {
                return;
            }
            let mob_entity = mob.get_mob_entity();
            let entity = &mob_entity.living_entity.entity;
            let world = entity.world.load();
            let position = entity.pos.load();
            let follow_range = mob_entity
                .living_entity
                .get_attribute_value(&Attributes::FOLLOW_RANGE);
            let entity_type = entity.entity_type;

            for nearby in world
                .get_nearby_entities(position, follow_range)
                .into_values()
            {
                if nearby.get_entity().entity_id == entity.entity_id
                    || nearby.get_entity().entity_type != entity_type
                {
                    continue;
                }
                let Some(nearby_mob) = nearby.get_mob() else {
                    continue;
                };
                if self.alert_only_adults && nearby.get_entity().age.load(Relaxed) < 0 {
                    continue;
                }
                if nearby_mob.get_mob_entity().target.lock().await.is_some() {
                    continue;
                }
                nearby_mob.set_mob_target(Some(target.clone())).await;
            }
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.target = None;
            self.track_target_goal.stop(mob).await;
        })
    }

    fn controls(&self) -> Controls {
        self.track_target_goal.controls()
    }
}

#[cfg(test)]
mod tests {
    use super::is_raider;
    use pumpkin_data::entity::EntityType;

    #[test]
    fn raid_mates_are_raiders() {
        assert!(is_raider(&EntityType::WITCH));
        assert!(is_raider(&EntityType::PILLAGER));
        assert!(is_raider(&EntityType::EVOKER));
        assert!(is_raider(&EntityType::RAVAGER));
        assert!(is_raider(&EntityType::VINDICATOR));
        assert!(is_raider(&EntityType::ILLUSIONER));
    }

    #[test]
    fn non_raiders_are_not_raiders() {
        // Vex is a raid participant but not tagged `#minecraft:raiders` in vanilla data,
        // matching `Vex` not extending `Raider` (it implements `OwnableEntity` instead).
        assert!(!is_raider(&EntityType::VEX));
        assert!(!is_raider(&EntityType::ZOMBIE));
        assert!(!is_raider(&EntityType::PLAYER));
    }
}
