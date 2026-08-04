use std::sync::Arc;
use std::sync::atomic::Ordering;

use super::{Controls, Goal};
use crate::entity::ai::goal::GoalFuture;
use crate::entity::mob::Mob;
use crate::entity::mob::creeper::CreeperEntity;

pub struct CreeperIgniteGoal {
    goal_control: Controls,
    creeper: Arc<CreeperEntity>,
}

impl CreeperIgniteGoal {
    #[must_use]
    pub const fn new(creeper: Arc<CreeperEntity>) -> Self {
        Self {
            goal_control: Controls::MOVE,
            creeper,
        }
    }

    const fn should_swell(distance_squared: f64, has_line_of_sight: bool) -> bool {
        distance_squared <= 49.0 && has_line_of_sight
    }

    async fn has_line_of_sight(mob: &dyn Mob, target: &dyn crate::entity::EntityBase) -> bool {
        let entity = mob.get_entity();
        entity
            .world
            .load_full()
            .raycast(
                entity.get_eye_pos(),
                target.get_entity().get_eye_pos(),
                async |block_pos, world| world.get_block_state(block_pos).is_solid(),
            )
            .await
            .is_none()
    }

    /// Vanilla `SwellGoal.canUse`: an already lit fuse keeps the goal alive regardless of
    /// distance, otherwise the swell only begins with a target inside 3 blocks.
    async fn can_swell(&self, mob: &dyn Mob) -> bool {
        if self.creeper.fuse_speed.load(Ordering::Relaxed) > 0 {
            return true;
        }

        let target_lock = mob.get_mob_entity().target.lock().await;
        if let Some(target) = target_lock.as_ref() {
            let dist_sq = mob
                .get_entity()
                .pos
                .load()
                .squared_distance_to_vec(&target.get_entity().pos.load());
            return dist_sq < 9.0;
        }

        false
    }
}

impl Goal for CreeperIgniteGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { self.can_swell(mob).await })
    }

    /// Vanilla `SwellGoal` inherits `canContinueToUse` from `Goal`, which returns `canUse()`.
    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { self.can_swell(mob).await })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
            navigator.stop();
        })
    }

    // Vanilla `SwellGoal.stop` only clears the cached target, it never resets the fuse.
    // We track the target through the mob itself, so there is nothing to clear here and
    // `tick` stays the single owner of the fuse speed.

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let target_lock = mob.get_mob_entity().target.lock().await;

            let Some(target) = target_lock.as_ref() else {
                self.creeper.set_fuse_speed(-1);
                return;
            };

            let dist_sq = mob
                .get_entity()
                .pos
                .load()
                .squared_distance_to_vec(&target.get_entity().pos.load());

            let has_line_of_sight = Self::has_line_of_sight(mob, target.as_ref()).await;
            if Self::should_swell(dist_sq, has_line_of_sight) {
                self.creeper.set_fuse_speed(1);
            } else {
                self.creeper.set_fuse_speed(-1);
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

#[cfg(test)]
mod tests {
    use super::CreeperIgniteGoal;

    #[test]
    fn swelling_requires_vanilla_distance_and_line_of_sight() {
        assert!(CreeperIgniteGoal::should_swell(49.0, true));
        assert!(!CreeperIgniteGoal::should_swell(49.0, false));
        assert!(!CreeperIgniteGoal::should_swell(49.000_001, true));
    }
}
