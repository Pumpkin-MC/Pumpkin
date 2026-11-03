use super::{Controls, Goal};
use crate::entity::EntityBase;
use crate::entity::mob::Mob;
use crate::entity::passive::tamable::TELEPORT_WHEN_DISTANCE_IS_SQ;
use std::sync::atomic::Ordering;

/// Holds an ordered-to-sit pet in place and in the sitting pose.
#[derive(Default)]
pub struct SitWhenOrderedToGoal {
    goal_control: Controls,
}

impl SitWhenOrderedToGoal {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            goal_control: Controls::MOVE.union(Controls::JUMP),
        }
    }

    fn set_pose(mob: &dyn Mob, sitting: bool) {
        if let Some(tamable) = mob.as_tamable() {
            tamable.set_in_sitting_pose(sitting);
        }
    }
}

impl Goal for SitWhenOrderedToGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let ordered = mob.ordered_to_sit();
        if !ordered && !mob.is_tamed() {
            return false;
        }

        let entity = &mob.get_mob_entity().living_entity.entity;
        if entity.touching_water.load(Ordering::Relaxed)
            || !entity.on_ground.load(Ordering::Relaxed)
        {
            return false;
        }

        let world = entity.world.load();
        let Some(owner) = mob
            .get_owner_uuid()
            .and_then(|uuid| world.get_player_by_uuid(uuid))
        else {
            return true;
        };
        // An owner under attack nearby makes the pet stand up.
        if entity
            .pos
            .load()
            .squared_distance_to_vec(&owner.get_entity().pos.load())
            < TELEPORT_WHEN_DISTANCE_IS_SQ
            && owner.living_entity.get_last_hurt_by_mob().is_some()
        {
            return false;
        }
        ordered
    }

    // Owner, water and ground only gate the start.
    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        mob.ordered_to_sit()
    }

    fn start(&mut self, mob: &dyn Mob) {
        mob.get_mob_entity().steered().navigator().stop();
        Self::set_pose(mob, true);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        Self::set_pose(mob, false);
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
