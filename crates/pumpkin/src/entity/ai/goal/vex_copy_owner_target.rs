use std::sync::Weak;

use crate::entity::EntityBase;
use crate::entity::ai::goal::{Controls, Goal, GoalFuture};
use crate::entity::mob::Mob;
use crate::entity::mob::vex::VexEntity;

/// Vanilla: `Vex.VexCopyOwnerTargetGoal`.
///
/// Adopts the owner's current target as this vex's own, approximating vanilla's
/// `TargetingConditions.forNonCombat().ignoreLineOfSight().ignoreInvisibilityTesting()`
/// predicate as "owner has a live target".
pub struct VexCopyOwnerTargetGoal {
    vex: Weak<VexEntity>,
}

impl VexCopyOwnerTargetGoal {
    #[must_use]
    pub const fn new(vex: Weak<VexEntity>) -> Self {
        Self { vex }
    }

    async fn owner_target(vex: &VexEntity) -> Option<std::sync::Arc<dyn EntityBase>> {
        let owner_id = vex.owner_id()?;
        let world = vex.mob_entity.living_entity.entity.world.load();
        let owner = world.get_entity_by_id(owner_id)?;
        let owner_mob = owner.get_mob()?;
        let target = owner_mob.get_mob_entity().target.lock().await.clone()?;
        target.get_entity().is_alive().then_some(target)
    }
}

impl Goal for VexCopyOwnerTargetGoal {
    fn can_start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(vex) = self.vex.upgrade() else {
                return false;
            };
            Self::owner_target(&vex).await.is_some()
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(vex) = self.vex.upgrade() else {
                return;
            };
            let target = Self::owner_target(&vex).await;
            mob.set_mob_target(target).await;
        })
    }

    fn controls(&self) -> Controls {
        Controls::TARGET
    }
}
