//! Vanilla 26.2 `MoveTowardsTargetGoal`.
//!
//! From `net.minecraft.world.entity.ai.goal.MoveTowardsTargetGoal`:
//! - Only runs when a target exists and distance² ≤ within²
//! - Picks a short path position *towards* the target (we approximate with
//!   a point partway to the target) and navigates there at `speed`.

use super::{Controls, Goal};
use crate::entity::ai::goal::GoalFuture;
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

pub struct MoveTowardsTargetGoal {
    speed: f64,
    /// Vanilla `within` — max distance from mob to target for this goal to run.
    within: f64,
    wanted: Option<Vector3<f64>>,
}

impl MoveTowardsTargetGoal {
    /// Vanilla iron golem: `MoveTowardsTargetGoal(this, 0.9, 32.0f)`.
    #[must_use]
    pub fn new(speed: f64, within: f64) -> Self {
        Self {
            speed: speed.max(0.01),
            within: within.max(1.0),
            wanted: None,
        }
    }
}

impl Goal for MoveTowardsTargetGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let target = mob.get_mob_entity().target.lock().await.clone();
            let Some(target) = target else {
                self.wanted = None;
                return false;
            };
            if let Some(living) = target.get_living_entity()
                && !living.is_alive()
            {
                self.wanted = None;
                return false;
            }
            let me = mob.get_entity().pos.load();
            let them = target.get_entity().pos.load();
            let dist_sq = me.squared_distance_to_vec(&them);
            let within_sq = self.within * self.within;
            if dist_sq > within_sq {
                self.wanted = None;
                return false;
            }
            // Vanilla: DefaultRandomPos.getPosTowards(mob, 16, 7, target.pos, π/2)
            // — random walkable point in a ±90° cone toward the target, range 16.
            // Approximate with a forward step along the target bearing (never
            // sideways/back): 8–16 blocks or 85% of remaining distance.
            let dx = them.x - me.x;
            let dz = them.z - me.z;
            let horiz = (dx * dx + dz * dz).sqrt();
            if horiz < 0.5 {
                self.wanted = None;
                return false;
            }
            let step = (8.0 + mob.get_random().random_range(0.0..8.0)).min(horiz * 0.85);
            let wanted = Vector3::new(
                me.x + dx / horiz * step,
                them.y.clamp(me.y - 7.0, me.y + 7.0),
                me.z + dz / horiz * step,
            );
            self.wanted = Some(wanted);
            true
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let target = mob.get_mob_entity().target.lock().await.clone();
            let Some(target) = target else {
                return false;
            };
            if let Some(living) = target.get_living_entity()
                && !living.is_alive()
            {
                return false;
            }
            let me = mob.get_entity().pos.load();
            let them = target.get_entity().pos.load();
            let within_sq = self.within * self.within;
            if me.squared_distance_to_vec(&them) >= within_sq {
                return false;
            }
            // Keep going while navigator still has a goal.
            !mob.get_mob_entity().navigator.lock().unwrap().is_idle()
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            let Some(wanted) = self.wanted else {
                return;
            };
            let mut nav = mob.get_mob_entity().navigator.lock().unwrap();
            nav.set_progress(NavigatorGoal {
                current_progress: mob.get_entity().pos.load(),
                destination: wanted,
                speed: self.speed,
            });
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.wanted = None;
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}
