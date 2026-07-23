use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering::Relaxed};

use super::{Controls, Goal};
use crate::entity::EntityBase;
use crate::entity::ai::goal::GoalFuture;
use crate::entity::ai::goal::track_target::TrackTargetGoal;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::mob::Mob;

/// Vanilla `HurtByTargetGoal` — retarget to whoever last hurt us.
///
/// Priority is higher than `ActiveTargetGoal`, so a nearby attacker that hits
/// the mob steals focus from a far opportunistic target. While running, if a
/// *new* attacker lands a hit, we switch to them (vanilla only sets last-hurt
/// mob; we re-acquire so "near zombie hitting me while I chase far one" works).
pub struct RevengeGoal {
    track_target_goal: TrackTargetGoal,
    target: Option<Arc<dyn EntityBase>>,
    last_attacked_time: AtomicI32,
    target_predicate: TargetPredicate,
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
            last_attacked_time: AtomicI32::new(0),
            target_predicate,
        }
    }

    fn resolve_attacker(
        &self,
        mob: &dyn Mob,
    ) -> Option<Arc<dyn EntityBase>> {
        let mob_entity = mob.get_mob_entity();
        let living = &mob_entity.living_entity;
        let attacker_id = living.last_attacker_id.load(Relaxed);
        if attacker_id == 0 {
            return None;
        }
        let world = living.entity.world.load();
        let attacker = world.get_entity_by_id(attacker_id).or_else(|| {
            world
                .get_player_by_id(attacker_id)
                .map(|p| p as Arc<dyn EntityBase>)
        })?;

        let attacker_living = attacker.get_living_entity()?;
        if !attacker_living.is_alive() {
            return None;
        }
        if !self
            .target_predicate
            .test(&world, Some(&mob_entity.living_entity), attacker_living)
        {
            return None;
        }
        // Don't revenge on self.
        if attacker.get_entity().entity_id == living.entity.entity_id {
            return None;
        }
        Some(attacker)
    }
}

impl Goal for RevengeGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let living = &mob.get_mob_entity().living_entity;
            let attacked_time = living.last_attacked_time.load(Relaxed);
            if attacked_time == self.last_attacked_time.load(Relaxed) {
                return false;
            }

            let Some(attacker) = self.resolve_attacker(mob) else {
                return false;
            };

            self.target = Some(attacker);
            true
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let living = &mob.get_mob_entity().living_entity;
            let attacked_time = living.last_attacked_time.load(Relaxed);

            // New hit while already revenging → switch to the freshest attacker
            // if they are still valid (near zombie while chasing far one).
            if attacked_time != self.last_attacked_time.load(Relaxed)
                && let Some(attacker) = self.resolve_attacker(mob)
            {
                let current_id = mob
                    .get_mob_entity()
                    .target
                    .lock()
                    .await
                    .as_ref()
                    .map(|t| t.get_entity().entity_id);
                if current_id != Some(attacker.get_entity().entity_id) {
                    mob.set_mob_target(Some(attacker)).await;
                }
                self.last_attacked_time.store(attacked_time, Relaxed);
            }

            self.track_target_goal.should_continue(mob).await
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            mob.set_mob_target(self.target.clone()).await;

            let mob_entity = mob.get_mob_entity();
            self.last_attacked_time.store(
                mob_entity.living_entity.last_attacked_time.load(Relaxed),
                Relaxed,
            );
            self.track_target_goal.max_time_without_visibility = 300;

            self.track_target_goal.start(mob).await;
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
