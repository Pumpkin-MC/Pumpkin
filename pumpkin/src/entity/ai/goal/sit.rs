use std::sync::atomic::Ordering;

use super::{Controls, Goal, GoalFuture};
use crate::entity::mob::Mob;

/// Goal for mobs (wolves, cats) to sit when ordered to do so.
///
/// Implements vanilla's `SitWhenOrderedToGoal` logic:
/// - Only active when ordered to sit
/// - Stops when the mob is no longer ordered to sit
/// - Prevents navigation/movement by controlling MOVE and JUMP
///
/// Simplifications vs vanilla (SitWhenOrderedToGoal.java:30-35):
/// - Owner-proximity and damage checks are omitted since Pumpkin's `owner` is just
///   a UUID, not a resolved entity. A complete implementation would require entity
///   lookup and level comparison, which is beyond the scope of this goal.
///
/// Note: Currently does not sync sitting pose to clients (`TAMEABLE_FLAGS` is not
/// actively tracked in Pumpkin for 26.1/26.2). The mob will be stationary but
/// render standing rather than in a sitting pose.
///
/// Vanilla source: `net/minecraft/world/entity/ai/goal/SitWhenOrderedToGoal.java`
pub struct SitGoal {
    goal_control: Controls,
}

impl Default for SitGoal {
    fn default() -> Self {
        Self {
            goal_control: Controls::MOVE | Controls::JUMP,
        }
    }
}

impl SitGoal {
    #[must_use]
    pub fn new() -> Box<Self> {
        Box::new(Self::default())
    }
}

impl Goal for SitGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let mob_entity = mob.get_mob_entity();
            let living = &mob_entity.living_entity;
            let entity = &living.entity;

            // Must be ordered to sit
            if !mob_entity.is_ordered_to_sit() {
                return false;
            }

            // Cannot sit in water
            if entity.touching_water.load(Ordering::SeqCst) {
                return false;
            }

            // Must be on ground
            if !entity.on_ground.load(Ordering::SeqCst) {
                return false;
            }

            true
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { mob.get_mob_entity().is_ordered_to_sit() })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            mob.get_mob_entity().navigator.lock().unwrap().stop();
        })
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
