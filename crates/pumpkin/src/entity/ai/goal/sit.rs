use std::sync::atomic::Ordering::Relaxed;

use super::{Controls, Goal, GoalFuture};
use crate::entity::mob::Mob;
use pumpkin_data::entity::EntityPose;

/// Vanilla `SitWhenOrderedToGoal` for tamable animals.
pub struct SitGoal {
    controls: Controls,
}

impl Default for SitGoal {
    fn default() -> Self {
        Self {
            controls: Controls::MOVE | Controls::JUMP,
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
            let entity = &mob.get_mob_entity().living_entity.entity;
            mob.is_tame()
                && mob.get_mob_entity().is_ordered_to_sit()
                && !entity.touching_water.load(Relaxed)
                && entity.on_ground.load(Relaxed)
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { mob.get_mob_entity().is_ordered_to_sit() })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            mob.get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .stop();
            mob.get_entity().set_pose(EntityPose::Sitting);
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            mob.get_entity().set_pose(EntityPose::Standing);
        })
    }

    fn controls(&self) -> Controls {
        self.controls
    }
}
