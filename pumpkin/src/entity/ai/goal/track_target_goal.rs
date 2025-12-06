use super::{Controls, Goal, to_goal_ticks};
use crate::entity::ai::goal::GoalFuture;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::living::LivingEntity;
use crate::entity::mob::Mob;
use crate::entity::{EntityBase, mob::MobEntity};
use rand::Rng;
use std::sync::Arc;

const UNSET: i32 = 0;
#[allow(dead_code)]
const CAN_TRACK: i32 = 1;
#[allow(dead_code)]
const CANNOT_TRACK: i32 = 2;

#[allow(dead_code)]
pub struct TrackTargetGoal {
    goal_control: Controls,
    target: Option<Arc<dyn EntityBase>>,
    check_visibility: bool,
    check_can_navigate: bool,
    can_navigate_flag: i32,
    check_can_navigate_cooldown: i32,
    time_without_visibility: i32,
    max_time_without_visibility: i32, // Default 60
}

#[allow(dead_code)]
impl TrackTargetGoal {
    #[must_use]
    pub fn new(check_visibility: bool, check_can_navigate: bool) -> Self {
        Self {
            goal_control: Controls::TARGET,
            target: None,
            check_visibility,
            check_can_navigate,
            can_navigate_flag: UNSET,
            check_can_navigate_cooldown: 0,
            time_without_visibility: 0,
            max_time_without_visibility: 60,
        }
    }

    pub fn with_default(check_visibility: bool) -> Self {
        Self::new(check_visibility, false)
    }

    // TODO: get from entity attribute
    pub fn get_follow_range(_mob: &MobEntity) -> f32 {
        1.0
    }

    fn can_navigate_to_entity(&mut self, mob: &dyn Mob, _target: &LivingEntity) -> bool {
        self.check_can_navigate_cooldown = to_goal_ticks(10 + mob.get_random().random_range(0..5));
        // TODO: after implementing path
        false
    }

    pub fn can_track(
        &mut self,
        mob: &dyn Mob,
        target: Option<&LivingEntity>,
        target_predicate: &TargetPredicate,
    ) -> bool {
        if target.is_none() {
            return false;
        }
        let mob_entity = mob.get_mob_entity();
        let target = target.unwrap();
        let world = &mob_entity.living_entity.entity.world;
        if !target_predicate.test(world.clone(), Some(&mob_entity.living_entity), target) {
            return false;
        } /*else if (!this.mob.isInPositionTargetRange(target.getBlockPos())) {
        return false;
        }*/
        // TODO: implement this

        if self.check_can_navigate {
            self.check_can_navigate_cooldown -= 1;
            if self.check_can_navigate_cooldown <= 0 {
                self.can_navigate_flag = UNSET;
            }

            if self.can_navigate_flag == UNSET {
                let value = if self.can_navigate_to_entity(mob, target) {
                    CAN_TRACK
                } else {
                    CANNOT_TRACK
                };
                self.can_navigate_flag = value;
            }

            if self.can_navigate_flag == CANNOT_TRACK {
                return false;
            }
        }

        true
    }
}

impl Goal for TrackTargetGoal {
    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let mob = mob.get_mob_entity();
            let mob_target = mob.target.lock().await;

            // We need to decide which target to use for the check
            let target = if mob_target.is_some() {
                (*mob_target).clone()
            } else {
                self.target.clone()
            };

            // Drop the guard immediately after access to release the lock
            drop(mob_target);

            if target.is_none() {
                return false;
            } // TODO: continue when scoreboard team are implemented
            true
        })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.can_navigate_flag = 0;
            self.check_can_navigate_cooldown = 0;
            self.time_without_visibility = 0;
            // No await needed here
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            let mob = mob.get_mob_entity();

            let mut mob_target = mob.target.lock().await;
            *mob_target = None;

            self.target = None;
        })
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
