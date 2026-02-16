use crate::entity::living::LivingEntity;
use crate::world::World;
use pumpkin_util::{Difficulty, GameMode};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub type TargetPredicateFilter =
    Arc<dyn Fn(&LivingEntity, &Arc<World>) -> Pin<Box<dyn Future<Output = bool> + Send>> + Send + Sync>;

pub struct TargetPredicate {
    pub base_max_distance: f64,
    pub attackable: bool,
    pub include_invulnerable: bool,
    pub use_line_of_sight: bool,
    pub predicate: Option<TargetPredicateFilter>,
}

impl TargetPredicate {
    pub async fn test(
        &self,
        world: &Arc<World>,
        tester: Option<&LivingEntity>,
        target: &LivingEntity,
    ) -> bool {
        // 1. Basic Check: Don't target yourself
        if let Some(tester) = tester {
            if Arc::ptr_eq(&tester.entity.arc, &target.entity.arc) {
                return false;
            }
        }

        // 2. Gamemode Check: AI ignores Creative and Spectator players
        let gamemode = target.entity.gamemode.load();
        if gamemode == GameMode::Creative || gamemode == GameMode::Spectator {
            return false;
        }

        // 3. Status Checks: Life, Invulnerability, and Difficulty
        if !target.is_alive() {
            return false;
        }

        if self.attackable {
            // Mobs don't attack in Peaceful difficulty
            if world.level_info.load().difficulty == Difficulty::Peaceful {
                return false;
            }

            // Check if target is invulnerable
            if !self.include_invulnerable && !target.can_take_damage() {
                return false;
            }
        }

        // 4. Distance Logic
        if let Some(tester) = tester {
            let mut max_dist = self.base_max_distance;

            // TODO: In Java, this pulls from GENERIC_FOLLOW_RANGE attribute.
            if max_dist  max_dist * max_dist {
                return false;
            }
        }

        // TODO: Implement Line of Sight (Raycasting) check

        // 5. Final custom filter
        if let Some(ref p) = self.predicate {
            if !p(target, world).await {
                return false;
            }
        }

        true
    }
}
