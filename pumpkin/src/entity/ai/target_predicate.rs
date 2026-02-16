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
        // 1. Equality check: An entity cannot target itself
        if let Some(tester_ent) = tester {
            if Arc::ptr_eq(&tester_ent.entity.arc, &target.entity.arc) {
                return false;
            }
        }

        // 2. Gamemode & State Checks: Ignore Spectators, Creative players, and Dead entities
        let gamemode = target.entity.gamemode.load();
        if gamemode == GameMode::Creative || gamemode == GameMode::Spectator || !target.is_alive() {
            return false;
        }

        // 3. Attackable Logic: Difficulty and Invulnerability
        if self.attackable {
            // Mobs do not target players in Peaceful difficulty
            if world.level_info.load().difficulty == Difficulty::Peaceful {
                return false;
            }
            // Minecraft check: Targets that cannot take damage are ignored by hostile AI
            if !target.can_take_damage() {
                return false;
            }
        }

        // 4. Distance Logic
        if let Some(tester_ent) = tester {
            let mut max_dist = self.base_max_distance;
            
            // If no specific distance is set, use the default follow range (16.0)
            if max_dist  (effective_max_dist * effective_max_dist) {
                return false;
            }
        }

        // 5. Custom Predicate Filter
        if let Some(ref p) = self.predicate {
            if !p(Arc::new(target.clone()), world.clone()).await {
                return false;
            }
        }

        true
    }
}
