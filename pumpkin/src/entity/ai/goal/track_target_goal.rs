use super::{Goal, GoalControl, to_goal_ticks};
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::living::LivingEntity;
use crate::entity::mob::Mob;
use crate::entity::{EntityBase, mob::MobEntity};
use async_trait::async_trait;
use rand::Rng;
use std::sync::Arc;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering::Relaxed;
use tokio::sync::Mutex;

const UNSET: i32 = 0;
#[allow(dead_code)]
const CAN_TRACK: i32 = 1;
#[allow(dead_code)]
const CANNOT_TRACK: i32 = 2;

#[allow(dead_code)]
pub struct TrackTargetGoal {
    goal_control: GoalControl,
    target: Mutex<Option<Arc<dyn EntityBase>>>,
    check_visibility: bool,
    check_can_navigate: bool,
    can_navigate_flag: AtomicI32,
    check_can_navigate_cooldown: AtomicI32,
    time_without_visibility: AtomicI32,
    max_time_without_visibility: AtomicI32, // Default 60
}

#[allow(dead_code)]
impl TrackTargetGoal {
    #[must_use]
    pub fn new(check_visibility: bool, check_can_navigate: bool) -> Self {
        Self {
            goal_control: GoalControl::default(),
            target: Mutex::new(None),
            check_visibility,
            check_can_navigate,
            can_navigate_flag: AtomicI32::new(UNSET),
            check_can_navigate_cooldown: AtomicI32::new(0),
            time_without_visibility: AtomicI32::new(0),
            max_time_without_visibility: AtomicI32::new(60),
        }
    }

    pub fn with_default(check_visibility: bool) -> Self {
        Self::new(check_visibility, false)
    }

    // TODO: get from entity attribute
    pub fn get_follow_range(_mob: &MobEntity) -> f64 {
        1.0
    }

    pub fn set_max_time_without_visibility(&self, time: i32) {
        self.max_time_without_visibility.store(time, Relaxed);
    }

    fn can_navigate_to_entity(&self, mob: &dyn Mob, _target: &LivingEntity) -> bool {
        self.check_can_navigate_cooldown.store(
            to_goal_ticks(10 + mob.get_random().random_range(0..5)),
            Relaxed,
        );
        // TODO: after implementing path
        false
    }

    pub async fn can_track(
        &self,
        mob: &dyn Mob,
        target: Option<&LivingEntity>,
        target_predicate: TargetPredicate,
    ) -> bool {
        if target.is_none() {
            return false;
        }
        let mob_entity = mob.get_mob_entity();
        let target = target.unwrap();
        let world = mob_entity.living_entity.entity.world.read().await;
        if !target_predicate.test(world.clone(), Some(&mob_entity.living_entity), target) {
            return false;
        } /*else if (!this.mob.isInPositionTargetRange(target.getBlockPos())) {
        return false;
        }*/
        // TODO: implement this
        drop(world); // Drop the lock because is useless now

        if self.check_can_navigate {
            if self.check_can_navigate_cooldown.fetch_sub(1, Relaxed) - 1 <= 0 {
                self.can_navigate_flag.store(UNSET, Relaxed);
            }

            if self.can_navigate_flag.load(Relaxed) == UNSET {
                let value = if self.can_navigate_to_entity(mob, target) {
                    CAN_TRACK
                } else {
                    CANNOT_TRACK
                };
                self.can_navigate_flag.store(value, Relaxed);
            }

            if self.can_navigate_flag.load(Relaxed) == CANNOT_TRACK {
                return false;
            }
        }

        true
    }
}

#[async_trait]
impl Goal for TrackTargetGoal {
    async fn can_start(&self, _mob: &dyn Mob) -> bool {
        false
    }

    async fn should_continue(&self, mob: &dyn Mob) -> bool {
        let mob = mob.get_mob_entity();
        let mob_target = mob.target.lock().await;
        let target = if mob_target.is_some() {
            mob_target.clone()
        } else {
            let lock = self.target.lock().await;
            lock.clone()
        };
        drop(mob_target);

        if target.is_none() {
            return false;
        } // TODO: continue when scoreboard team are implemented
        false
    }

    async fn start(&self, _mob: &dyn Mob) {
        self.can_navigate_flag.store(0, Relaxed);
        self.check_can_navigate_cooldown.store(0, Relaxed);
        self.time_without_visibility.store(0, Relaxed);
    }

    async fn stop(&self, mob: &dyn Mob) {
        let mob = mob.get_mob_entity();
        *mob.target.lock().await = None;
        *self.target.lock().await = None;
    }

    async fn tick(&self, _mob: &dyn Mob) {}

    fn get_goal_control(&self) -> &GoalControl {
        &self.goal_control
    }
}
