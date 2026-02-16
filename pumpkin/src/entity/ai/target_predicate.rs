use crate::entity::living::LivingEntity;
use crate::world::World;
use pumpkin_util::{Difficulty, GameMode};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

const MIN_DISTANCE: f64 = 2.0;

pub type PredicateFn = dyn Fn(Arc<LivingEntity>, Arc<World>) -> Pin<Box<dyn Future<Output = bool> + Send>>
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

    pub async fn test(
        &self,
        world: &Arc<World>,
        tester: Option<&LivingEntity>,
        target: &LivingEntity,
    ) -> bool {
        if let Some(t) = tester {
            if Arc::ptr_eq(&t.entity.arc, &target.entity.arc) {
                return false;
            }
        }

        let gm = target.entity.gamemode.load();
        if gm == GameMode::Creative || gm == GameMode::Spectator || !target.is_alive() {
            return false;
        }

        if self.attackable {
            let diff = world.level_info.load().difficulty;
            if diff == Difficulty::Peaceful || !target.can_take_damage() {
                return false;
            }
        }

        if let Some(t_ent) = tester {
            let d_sq = t_ent
                .entity
                .pos
                .load()
                .distance_squared(target.entity.pos.load());
            let mut range = self.base_max_distance;
            if range < 0.0 {
                range = 16.0;
            }
            if d_sq > (range * range).max(MIN_DISTANCE * MIN_DISTANCE) {
                return false;
            }
        }

        if let Some(ref p) = self.predicate {
            if !p(Arc::new(target.clone()), world.clone()).await {
                return false;
            }
        }

        true
    }
}
