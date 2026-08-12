use pumpkin_util::Difficulty;

use crate::entity::{EntityBase, living::LivingEntity};
use crate::world::World;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

const MIN_DISTANCE: f64 = 2.0;

pub type PredicateFn = dyn for<'a> Fn(&'a LivingEntity, &'a World) -> Pin<Box<dyn Future<Output = bool> + Send + 'a>>
    + Send
    + Sync;

pub struct TargetPredicate {
    pub attackable: bool,
    pub base_max_distance: f64,
    pub respects_visibility: bool,
    pub use_distance_scaling_factor: bool,
    pub predicate: Option<Arc<PredicateFn>>,
}

impl Default for TargetPredicate {
    fn default() -> Self {
        Self {
            attackable: true,
            base_max_distance: -1.0,
            respects_visibility: true,
            use_distance_scaling_factor: true,
            predicate: None,
        }
    }
}

impl TargetPredicate {
    fn new(attackable: bool) -> Self {
        Self {
            attackable,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn create_attackable() -> Self {
        Self::new(true)
    }

    #[must_use]
    pub fn create_non_attackable() -> Self {
        Self::new(false)
    }

    #[must_use]
    pub fn copy(&self) -> Self {
        Self {
            attackable: self.attackable,
            base_max_distance: self.base_max_distance,
            respects_visibility: self.respects_visibility,
            use_distance_scaling_factor: self.use_distance_scaling_factor,
            predicate: self.predicate.clone(),
        }
    }

    #[must_use]
    pub const fn set_base_max_distance(mut self, base_max_distance: f64) -> Self {
        self.base_max_distance = base_max_distance;
        self
    }

    #[must_use]
    pub const fn ignore_visibility(mut self) -> Self {
        self.respects_visibility = false;
        self
    }

    #[must_use]
    pub const fn ignore_distance_scaling_factor(mut self) -> Self {
        self.use_distance_scaling_factor = false;
        self
    }

    pub fn set_predicate<F>(&mut self, predicate: F)
    where
        F: for<'a> Fn(
                &'a LivingEntity,
                &'a World,
            ) -> Pin<Box<dyn Future<Output = bool> + Send + 'a>>
            + Send
            + Sync
            + 'static,
    {
        self.predicate = Some(Arc::new(predicate));
    }

    pub async fn test(
        &self,
        world: &World,
        tester: Option<&LivingEntity>,
        target: &LivingEntity,
    ) -> bool {
        if tester.is_some_and(|t| std::ptr::eq(t, target)) {
            return false;
        }

        if !target.is_alive() {
            return false;
        }

        let targeter_base =
            tester.and_then(|tester_ent| world.get_entity_by_uuid(tester_ent.entity.entity_uuid));

        if let Some(tester_ent) = tester
            && self.attackable
        {
            let can_attack = targeter_base
                .as_deref()
                .and_then(EntityBase::get_mob)
                .map_or_else(
                    || tester_ent.can_attack_target(target, world),
                    |mob| mob.can_attack(target),
                );
            if !can_attack
                || world
                    .are_allied(targeter_base.as_deref().unwrap_or(tester_ent), target)
                    .await
            {
                return false;
            }
        } else if self.attackable
            && (!target.can_be_seen_as_enemy()
                || world.level_info.load().difficulty == Difficulty::Peaceful)
        {
            return false;
        }

        if let Some(tester_ent) = tester
            && self.base_max_distance > 0.0
        {
            let visibility = if self.use_distance_scaling_factor {
                target.visibility_percent(tester_ent).await
            } else {
                1.0
            };
            let max_dist = (self.base_max_distance * visibility).max(MIN_DISTANCE);
            let dist_sq = tester_ent
                .entity
                .pos
                .load()
                .squared_distance_to_vec(&target.entity.pos.load());

            if dist_sq > max_dist * max_dist {
                return false;
            }
        }

        if self.respects_visibility
            && let Some(tester_ent) = tester
            && tester_ent
                .entity
                .world
                .load_full()
                .raycast(
                    tester_ent.entity.get_eye_pos(),
                    target.entity.get_eye_pos(),
                    async |block_pos, world| world.get_block_state(block_pos).is_solid(),
                )
                .await
                .is_some()
        {
            return false;
        }

        if let Some(predicate) = &self.predicate
            && !predicate(target, world).await
        {
            return false;
        }

        true
    }

    pub async fn test_custom_predicate(&self, world: &World, target: &LivingEntity) -> bool {
        match &self.predicate {
            Some(predicate) => predicate(target, world).await,
            None => true,
        }
    }
}
