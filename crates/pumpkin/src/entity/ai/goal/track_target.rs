use super::{Controls, Goal, to_goal_ticks};

use crate::entity::EntityBase;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::mob::Mob;
use pumpkin_data::attributes::Attributes;
use rand::RngExt;
use std::sync::Arc;

const UNSET: i32 = 0;
const CAN_TRACK: i32 = 1;
const CANNOT_TRACK: i32 = 2;

pub struct TrackTargetGoal {
    goal_control: Controls,
    check_visibility: bool,
    check_can_navigate: bool,
    can_navigate_flag: i32,
    check_can_navigate_cooldown: i32,
    time_without_visibility: i32,
    pub max_time_without_visibility: i32,
    /// Fallback target, used while the mob's own target is cleared.
    target_mob: Option<Arc<dyn EntityBase>>,
}

#[expect(dead_code)]
impl TrackTargetGoal {
    #[must_use]
    pub fn new(check_visibility: bool, check_can_navigate: bool) -> Self {
        Self {
            goal_control: Controls::TARGET,
            check_visibility,
            check_can_navigate,
            can_navigate_flag: UNSET,
            check_can_navigate_cooldown: 0,
            time_without_visibility: 0,
            max_time_without_visibility: 60,
            target_mob: None,
        }
    }

    pub fn with_default(check_visibility: bool) -> Self {
        Self::new(check_visibility, false)
    }

    pub const fn set_unseen_memory_ticks(mut self, ticks: i32) -> Self {
        self.max_time_without_visibility = ticks;
        self
    }

    /// Set by goals that already know their target.
    pub fn set_target_mob(&mut self, target: Option<Arc<dyn EntityBase>>) {
        self.target_mob = target;
    }

    fn can_navigate_to_entity(&mut self, mob: &dyn Mob) -> bool {
        self.check_can_navigate_cooldown = to_goal_ticks(10 + mob.get_random().random_range(0..5));
        // TODO: after implementing path
        false
    }

    const fn remembers_visible_target(&mut self, has_line_of_sight: bool) -> bool {
        if has_line_of_sight {
            self.time_without_visibility = 0;
            true
        } else {
            self.time_without_visibility += 1;
            self.time_without_visibility <= to_goal_ticks(self.max_time_without_visibility)
        }
    }

    /// Targeting check goals run inside `can_start`.
    pub fn can_track(
        &mut self,
        mob: &dyn Mob,
        target: Option<&dyn EntityBase>,
        target_predicate: &TargetPredicate,
    ) -> bool {
        let Some(target) = target else {
            return false;
        };

        let mob_entity = mob.get_mob_entity();
        let world = mob_entity.living_entity.entity.world.load();

        if !target_predicate.test(&world, Some(mob), target) {
            return false;
        }

        // TODO: isInPositionTargetRange (isWithinHome in Java) check

        if self.check_can_navigate {
            self.check_can_navigate_cooldown -= 1;
            if self.check_can_navigate_cooldown <= 0 {
                self.can_navigate_flag = UNSET;
            }

            if self.can_navigate_flag == UNSET {
                self.can_navigate_flag = if self.can_navigate_to_entity(mob) {
                    CAN_TRACK
                } else {
                    CANNOT_TRACK
                };
            }

            if self.can_navigate_flag == CANNOT_TRACK {
                return false;
            }
        }

        true
    }
}

impl Goal for TrackTargetGoal {
    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let mob_entity = mob.get_mob_entity();
        let Some(target_base) = mob_entity.get_target().or_else(|| self.target_mob.clone()) else {
            return false;
        };

        let Some(target) = target_base.get_living_entity() else {
            return false;
        };

        if !mob_entity.living_entity.can_attack(target) {
            return false;
        }

        let mob_base: &dyn EntityBase = mob;
        if mob_base.is_allied_to(target_base.as_ref()) {
            return false;
        }

        let dist_sq = mob_entity
            .living_entity
            .entity
            .pos
            .load()
            .squared_distance_to_vec(&target.entity.pos.load());

        // Get follow range attribute value and check if target is within range
        let follow_range = mob_entity
            .living_entity
            .get_attribute_value(&Attributes::FOLLOW_RANGE);

        if dist_sq > follow_range * follow_range {
            return false;
        }

        if self.check_visibility {
            let has_line_of_sight = mob.has_line_of_sight(&target.entity);

            if !self.remembers_visible_target(has_line_of_sight) {
                return false;
            }
        }

        mob.set_mob_target(Some(target_base.clone()));
        true
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.can_navigate_flag = UNSET;
        self.check_can_navigate_cooldown = 0;
        self.time_without_visibility = 0;
    }

    fn stop(&mut self, mob: &dyn Mob) {
        mob.set_mob_target(None);
        self.target_mob = None;
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

#[cfg(test)]
mod tests {
    use super::{TrackTargetGoal, to_goal_ticks};

    #[test]
    fn forgets_unseen_target_after_vanilla_memory_window() {
        let mut goal = TrackTargetGoal::with_default(true);
        let memory_ticks = to_goal_ticks(goal.max_time_without_visibility);

        for _ in 0..memory_ticks {
            assert!(goal.remembers_visible_target(false));
        }
        assert!(!goal.remembers_visible_target(false));
    }

    #[test]
    fn seeing_target_resets_unseen_memory() {
        let mut goal = TrackTargetGoal::with_default(true);
        assert!(goal.remembers_visible_target(false));
        assert!(goal.remembers_visible_target(true));
        assert_eq!(goal.time_without_visibility, 0);
    }
}
