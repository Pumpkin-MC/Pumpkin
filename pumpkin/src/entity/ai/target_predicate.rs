use crate::entity::EntityBase;
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

    pub fn set_predicate<F, Fut>(&mut self, predicate: F)
    where
        F: Fn(Arc<LivingEntity>, Arc<World>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = bool> + Send + 'static,
    {
        self.predicate = Some(Arc::new(
            move |living_entity: Arc<LivingEntity>, world: Arc<World>| {
                Box::pin(predicate(living_entity, world))
            },
        ));
    }

    pub fn test(
        &self,
        world: &World,
        tester: Option<&LivingEntity>,
        target: &LivingEntity,
    ) -> bool {
        // 1. Equality check: A mob cannot target itself
        if let Some(t) = tester
            && t.entity.entity_id == target.entity.entity_id
        {
            return false;
        }

        // 2. Vanilla Gamemode Check: AI ignores Creative and Spectator players
        if let Some(player) = target.entity.get_player() {
            let gm = player.gamemode.load();
            if gm == GameMode::Creative || gm == GameMode::Spectator {
                return false;
            }
        }

        // 3. Status Checks: Life, Invulnerability, and Difficulty
        if !target.entity.is_alive() {
            return false;
        }

        if self.attackable {
            // Mobs don't attack in Peaceful difficulty
            if world.level_info.load().difficulty == Difficulty::Peaceful {
                return false;
            }
            // Ignore targets that can't take damage (matches vanilla TargetPredicate)
            if !target.can_take_damage() {
                return false;
            }
        }

        // 4. Distance Logic
        if let Some(t_ent) = tester {
            let p1 = t_ent.entity.pos.load();
            let p2 = target.entity.pos.load();
            let d_sq = p1.squared_distance_to_vec(&p2);

            let mut dist_limit = self.base_max_distance;
            if dist_limit < 0.0 {
                dist_limit = 16.0; // Vanilla default follow range
            }

            // TODO: Visibility Modifier Logic (Sneaking = 0.8x, Invisibility = 0.07x)
            // Minecraft logic: max_dist *= target.get_visibility_modifier(tester);

            let final_limit = dist_limit * dist_limit;
            let min_limit = MIN_DISTANCE * MIN_DISTANCE;

            if d_sq > final_limit.max(min_limit) {
                return false;
            }
        }

        // TODO: Implement Line of Sight (Raycasting) check if self.use_line_of_sight is true
        // Minecraft uses: if (this.useLineOfSight && !tester.getSensing().canSee(target)) return false;

        true
    }
}
