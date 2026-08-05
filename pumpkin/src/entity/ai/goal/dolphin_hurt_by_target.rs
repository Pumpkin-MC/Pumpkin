use std::sync::atomic::Ordering::Relaxed;

use super::{Controls, Goal, GoalFuture};
use crate::entity::ai::goal::revenge::RevengeGoal;
use crate::entity::mob::Mob;
use pumpkin_data::entity::EntityType;

/// `Dolphin.registerGoals` (`Dolphin.java:171`): `new HurtByTargetGoal(this,
/// Guardian.class).setAlertOthers()`.
///
/// `HurtByTargetGoal`'s vararg constructor parameter is `ignoreDamageFromTheseTypes`
/// (`HurtByTargetGoal.java:26-30,32-48`), not a restriction to only retaliate against that type:
/// `canUse` returns `false` whenever the attacker's class is assignable from one of the ignored
/// types. So dolphins retaliate against anything that hurts them *except* a `Guardian` (matching
/// `AvoidEntityGoal`'s flee-from-guardian behavior at priority 9 instead of fighting it).
/// `isAssignableFrom` also covers `ElderGuardian extends Guardian`, so both entity types are
/// excluded here.
///
/// This codebase's `RevengeGoal` has no ignore-type list, so this wraps it with the extra
/// pre-check, following the `LlamaHurtByTargetGoal` wrapper pattern in
/// `llama_hurt_by_target.rs`. `setAlertOthers()` with no exception types leaves vanilla's
/// `toIgnoreAlert` non-null but empty, i.e. every same-species mob within follow range gets
/// alerted -- exactly what `RevengeGoal::start` already does unconditionally, so no extra work
/// is needed for that half.
pub struct DolphinHurtByTargetGoal {
    inner: RevengeGoal,
}

const IGNORED_ATTACKER_TYPES: &[&EntityType] =
    &[&EntityType::GUARDIAN, &EntityType::ELDER_GUARDIAN];

impl DolphinHurtByTargetGoal {
    #[must_use]
    pub fn new() -> Box<Self> {
        Box::new(Self {
            inner: RevengeGoal::new(true),
        })
    }
}

impl Goal for DolphinHurtByTargetGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let living = &mob.get_mob_entity().living_entity;
            let attacker_id = living.last_attacker_id.load(Relaxed);
            if attacker_id != 0 {
                let world = living.entity.world.load();
                if let Some(attacker) = world.get_entity_by_id(attacker_id) {
                    let attacker_type = attacker.get_entity().entity_type;
                    if IGNORED_ATTACKER_TYPES.contains(&attacker_type) {
                        return false;
                    }
                }
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

    fn controls(&self) -> Controls {
        self.inner.controls()
    }
}
