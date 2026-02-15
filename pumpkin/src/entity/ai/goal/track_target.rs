use super::{Controls, Goal, to_goal_ticks};
use crate::entity::ai::goal::GoalFuture;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::living::LivingEntity;
use crate::entity::mob::Mob;
use crate::entity::mob::MobEntity;
use rand::RngExt;

const UNSET: i32 = 0;
const CAN_TRACK: i32 = 1;
const CANNOT_TRACK: i32 = 2;

#[expect(dead_code)]
pub struct TrackTargetGoal {
    goal_control: Controls,
    check_visibility: bool,
    check_can_navigate: bool,
    can_navigate_flag: i32,
    check_can_navigate_cooldown: i32,
    time_without_visibility: i32,
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
            can_navigate_flag: UNSET,
            check_can_navigate_cooldown: 0,
            time_without_visibility: 0,
            max_time_without_visibility: 60,
            target_predicate: TargetPredicate::create_attackable(),
        }
    }

    pub fn with_default(check_visibility: bool) -> Self {
        Self::new(check_visibility, false)
    }

    // TODO: get from entity attribute (enderman=64, zombie=35, default=16)
    pub const fn get_follow_range(_mob: &MobEntity) -> f64 {
        64.0
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
        let world = mob_entity.living_entity.entity.world.load_full();
        if !target_predicate.test(&world, Some(&mob_entity.living_entity), target) {
            return false;
        }
        // TODO: isInPositionTargetRange check

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
            let mob_entity = mob.get_mob_entity();
            let target_arc = mob_entity.target.lock().await.clone();

            let Some(target_base) = target_arc else {
                return false;
            };

            let Some(target) = target_base.get_living_entity() else {
                return false;
            };

            if !target.entity.is_alive() {
                return false;
            }

            let dist_sq = mob_entity
                .living_entity
                .entity
                .pos
                .load()
                .squared_distance_to_vec(&target.entity.pos.load());
            let follow_range = Self::get_follow_range(mob_entity);
            if dist_sq > follow_range * follow_range {
                return false;
            }

            // TODO: Visibility timeout (check_visibility flag)

            true
        })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.can_navigate_flag = 0;
            self.check_can_navigate_cooldown = 0;
            self.time_without_visibility = 0;
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            mob.set_mob_target(None).await;
        })
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
