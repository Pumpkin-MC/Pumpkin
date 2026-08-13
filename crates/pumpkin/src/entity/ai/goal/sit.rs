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
            let ordered_to_sit = mob.get_mob_entity().is_ordered_to_sit();
            if !ordered_to_sit && !mob.is_tame() {
                return false;
            }
            if entity.touching_water.load(Relaxed) || !entity.on_ground.load(Relaxed) {
                return false;
            }

            let Some(owner_uuid) = mob.get_owner_uuid() else {
                return true;
            };
            let world = entity.world.load_full();
            let Some(owner) = world.get_player_by_uuid(owner_uuid) else {
                return true;
            };
            let distance_sq = entity
                .pos
                .load()
                .squared_distance_to_vec(&owner.entity.pos.load());
            let owner_was_hurt = owner.living_entity.last_attacker_id.load(Relaxed) != 0;
            !(distance_sq < 144.0 && owner_was_hurt) && ordered_to_sit
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
