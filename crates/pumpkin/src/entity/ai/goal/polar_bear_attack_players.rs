use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;

use super::{Controls, Goal, GoalFuture};
use crate::entity::ai::goal::active_target::ActiveTargetGoal;
use crate::entity::living::LivingEntity;
use crate::entity::mob::{Mob, MobEntity};
use crate::world::World;
use pumpkin_data::entity::EntityType;

/// `PolarBear.PolarBearAttackPlayersGoal` (PolarBear.java:254-280): an adult bear targets
/// nearby players only while a cub is nearby.
///
/// Vanilla checks an `inflate(8.0, 4.0, 8.0)` box;
/// approximated here by querying an 8-block radius sphere (since `World` only exposes
/// radius-based entity queries) and re-applying the box's vertical half-range of 4.0 per
/// candidate. The `getFollowDistance() * 0.5` override (forget the target sooner once
/// tracking) has no hook point in `TrackTargetGoal` and is not ported.
pub struct PolarBearAttackPlayersGoal {
    inner: ActiveTargetGoal,
}

impl PolarBearAttackPlayersGoal {
    #[must_use]
    pub fn new(mob: &MobEntity) -> Box<Self> {
        Box::new(Self {
            inner: ActiveTargetGoal::new(
                mob,
                &EntityType::PLAYER,
                20,
                true,
                true,
                Some(|_target: Arc<LivingEntity>, _world: Arc<World>| async move { true }),
            ),
        })
    }
}

impl Goal for PolarBearAttackPlayersGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if mob.get_entity().age.load(Relaxed) < 0 {
                return false;
            }
            if !self.inner.can_start(mob).await {
                return false;
            }

            let mob_entity = mob.get_mob_entity();
            let pos = mob_entity.living_entity.entity.pos.load();
            let world = mob_entity.living_entity.entity.world.load();
            world.get_nearby_entities(pos, 8.0).values().any(|nearby| {
                let other = nearby.get_entity();
                other.entity_type == &EntityType::POLAR_BEAR
                    && other.age.load(Relaxed) < 0
                    && (other.pos.load().y - pos.y).abs() <= 4.0
            })
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
