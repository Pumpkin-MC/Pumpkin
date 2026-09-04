use pumpkin_util::Difficulty;

use crate::entity::EntityBase;
use crate::entity::living::LivingEntity;
use crate::world::World;
use std::sync::Arc;

const MIN_DISTANCE: f64 = 2.0;

pub type PredicateFn = dyn Fn(&LivingEntity, &World) -> bool + Send + Sync;

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
        F: Fn(&LivingEntity, &World) -> bool + Send + Sync + 'static,
    {
        self.predicate = Some(Arc::new(predicate));
    }

    pub fn test(
        &self,
        world: &World,
        tester: Option<&dyn EntityBase>,
        target: &dyn EntityBase,
    ) -> bool {
        let Some(target_living) = target.get_living_entity() else {
            return false;
        };

        if tester.is_some_and(|t| t.get_entity().entity_id == target.get_entity().entity_id) {
            return false;
        }

        if !target_living.is_part_of_game() {
            return false;
        }

        if let Some(predicate) = &self.predicate
            && !predicate(target_living, world)
        {
            return false;
        }

        let Some(tester) = tester else {
            // Without a tester only the plain "can be seen as enemy" rule applies.
            return !(self.attackable
                && (!target_living.can_take_damage()
                    || world.level_info.load().difficulty == Difficulty::Peaceful));
        };
        let Some(tester_living) = tester.get_living_entity() else {
            return false;
        };

        if self.attackable
            && (!tester_living.can_attack(target_living) || tester.is_allied_to(target))
        {
            return false;
        }

        if self.base_max_distance > 0.0 {
            // TODO: use distance_scaling_factor from target
            let max_dist = self.base_max_distance.max(MIN_DISTANCE);
            let dist_sq = tester
                .get_entity()
                .pos
                .load()
                .squared_distance_to_vec(&target.get_entity().pos.load());

            if dist_sq > max_dist * max_dist {
                return false;
            }
        }

        if self.respects_visibility
            && let Some(mob) = tester.get_mob()
            && !mob.has_line_of_sight(target.get_entity())
        {
            return false;
        }

        true
    }
}
