use super::{Controls, Goal, to_goal_ticks};

use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::living::LivingEntity;
use crate::entity::mob::Mob;
use pumpkin_data::attributes::Attributes;
use rand::RngExt;
use std::sync::atomic::{AtomicI32, Ordering};

const UNSET: i32 = 0;
const CAN_TRACK: i32 = 1;
const CANNOT_TRACK: i32 = 2;

pub struct TrackTargetGoal {
    goal_control: Controls,
    check_visibility: bool,
    check_can_navigate: bool,
    can_navigate_flag: AtomicI32,
    check_can_navigate_cooldown: AtomicI32,
    time_without_visibility: AtomicI32,
    pub max_time_without_visibility: i32,
    target_predicate: TargetPredicate,
}

#[expect(dead_code)]
impl TrackTargetGoal {
    #[must_use]
    pub fn new(check_visibility: bool, check_can_navigate: bool) -> Self {
        Self {
            goal_control: Controls::TARGET,
            check_visibility,
            check_can_navigate,
            can_navigate_flag: AtomicI32::new(UNSET),
            check_can_navigate_cooldown: AtomicI32::new(0),
            time_without_visibility: AtomicI32::new(0),
            max_time_without_visibility: 60,
            target_predicate: TargetPredicate::create_attackable().ignore_visibility(),
        }
    }

    pub fn with_default(check_visibility: bool) -> Self {
        Self::new(check_visibility, false)
    }

    pub const fn set_unseen_memory_ticks(mut self, ticks: i32) -> Self {
        self.max_time_without_visibility = ticks;
        self
    }

    fn can_navigate_to_entity(&self, mob: &dyn Mob, _target: &LivingEntity) -> bool {
        let cooldown = to_goal_ticks(10 + mob.get_random().random_range(0..5));
        self.check_can_navigate_cooldown
            .store(cooldown, Ordering::Relaxed);
        // TODO: after implementing path
        false
    }

    fn remembers_visible_target(&self, has_line_of_sight: bool) -> bool {
        if has_line_of_sight {
            self.time_without_visibility.store(0, Ordering::Relaxed);
            true
        } else {
            let unseen_ticks = self.time_without_visibility.fetch_add(1, Ordering::Relaxed) + 1;
            unseen_ticks <= to_goal_ticks(self.max_time_without_visibility)
        }
    }

    /// Equivalent to Vanilla's `canAttack` check inside `TargetGoal`
    pub fn can_track(
        &self,
        mob: &dyn Mob,
        target: Option<&LivingEntity>,
        target_predicate: &TargetPredicate,
    ) -> bool {
        let Some(target) = target else {
            return false;
        };

        let mob_entity = mob.get_mob_entity();
        let world = mob_entity.living_entity.entity.world.load();

        if !target_predicate.test(&world, Some(&mob_entity.living_entity), target) {
            return false;
        }

        // TODO: isInPositionTargetRange (isWithinHome in Java) check

        if self.check_can_navigate {
            let cooldown = self
                .check_can_navigate_cooldown
                .fetch_sub(1, Ordering::Relaxed)
                - 1;
            if cooldown <= 0 {
                self.can_navigate_flag.store(UNSET, Ordering::Relaxed);
            }

            if self.can_navigate_flag.load(Ordering::Relaxed) == UNSET {
                let can_reach = self.can_navigate_to_entity(mob, target);
                self.can_navigate_flag.store(
                    if can_reach { CAN_TRACK } else { CANNOT_TRACK },
                    Ordering::Relaxed,
                );
            }

            if self.can_navigate_flag.load(Ordering::Relaxed) == CANNOT_TRACK {
                return false;
            }
        }

        true
    }
}

impl Goal for TrackTargetGoal {
    fn should_continue(&self, mob: &dyn Mob) -> bool {
        let mob_entity = mob.get_mob_entity();
        let target_arc = mob_entity.get_target();

        let Some(target_base) = target_arc else {
            return false;
        };

        let Some(target) = target_base.get_living_entity() else {
            return false;
        };

        if !target.entity.is_alive() {
            return false;
        }

        if !self.can_track(mob, Some(target), &self.target_predicate) {
            return false;
        }

        // TODO: Team checks (return false if on the same team)

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
            let world = mob_entity.living_entity.entity.world.load();
            let has_line_of_sight = world
                .raycast(
                    mob_entity.living_entity.entity.get_eye_pos(),
                    target.entity.get_eye_pos(),
                    |block_pos, world| world.get_block_state(block_pos).is_solid(),
                )
                .is_none();

            if !self.remembers_visible_target(has_line_of_sight) {
                return false;
            }
        }

        mob.set_mob_target(Some(target_base.clone()));
        true
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.can_navigate_flag.store(UNSET, Ordering::Relaxed);
        self.check_can_navigate_cooldown.store(0, Ordering::Relaxed);
        self.time_without_visibility.store(0, Ordering::Relaxed);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        mob.set_mob_target(None);
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

#[cfg(test)]
mod tests {
    use super::{TrackTargetGoal, to_goal_ticks};
    use std::sync::atomic::Ordering;

    #[test]
    fn forgets_unseen_target_after_vanilla_memory_window() {
        let goal = TrackTargetGoal::with_default(true);
        let memory_ticks = to_goal_ticks(goal.max_time_without_visibility);

        for _ in 0..memory_ticks {
            assert!(goal.remembers_visible_target(false));
        }
        assert!(!goal.remembers_visible_target(false));
    }

    #[test]
    fn seeing_target_resets_unseen_memory() {
        let goal = TrackTargetGoal::with_default(true);
        assert!(goal.remembers_visible_target(false));
        assert!(goal.remembers_visible_target(true));
        assert_eq!(goal.time_without_visibility.load(Ordering::Relaxed), 0);
    }
}
