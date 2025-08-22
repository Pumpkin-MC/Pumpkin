use super::{Control, Goal, GoalControl, to_goal_ticks};
use crate::entity::mob::Mob;
use crate::world::World;
use async_trait::async_trait;
use crossbeam::atomic::AtomicCell;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::Rng;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicI32};
use std::sync::{Arc, Weak};

const MIN_WAITING_TIME: i32 = 1200;
const MAX_TRYING_TIME: i32 = 1200;
const MIN_INTERVAL: i32 = 200;

#[allow(dead_code)]
pub struct MoveToTargetPosGoal {
    goal_control: GoalControl,
    move_to_target_pos: Weak<dyn MoveToTargetPos>,
    pub speed: f64,
    pub cooldown: AtomicI32,
    pub trying_time: AtomicI32,
    pub safe_waiting_time: AtomicI32,
    pub target_pos: AtomicCell<BlockPos>,
    pub reached: AtomicBool,
    pub range: i32,
    pub max_y_difference: i32,
    pub lowest_y: AtomicI32,
}

impl MoveToTargetPosGoal {
    #[must_use]
    pub fn new(
        move_to_target_pos: Weak<dyn MoveToTargetPos>,
        speed: f64,
        range: i32,
        max_y_difference: i32,
    ) -> Self {
        Self {
            goal_control: GoalControl::from_array(&[Control::Move, Control::Jump]),
            move_to_target_pos,
            speed,
            cooldown: AtomicI32::new(0),
            trying_time: AtomicI32::new(0),
            safe_waiting_time: AtomicI32::new(0),
            target_pos: AtomicCell::new(BlockPos::new(0, 0, 0)),
            reached: AtomicBool::new(false),
            range,
            max_y_difference,
            lowest_y: AtomicI32::new(0),
        }
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn with_default(
        move_to_target_pos: Weak<dyn MoveToTargetPos>,
        speed: f64,
        range: i32,
    ) -> Self {
        Self::new(move_to_target_pos, speed, range, 1)
    }

    pub fn get_interval(mob: &dyn Mob) -> i32 {
        to_goal_ticks(MIN_INTERVAL + mob.get_random().random_range(0..MIN_INTERVAL))
    }

    pub async fn find_target_pos(&self, mob: &dyn Mob) -> bool {
        let block_pos = mob.get_entity().block_pos.load();
        let mut block_pos_mutable = BlockPos::new(0, 0, 0);

        let mut k = self.lowest_y.load(Relaxed);
        while k <= self.max_y_difference {
            for l in 0..self.range {
                let mut m = 0;
                while m <= l {
                    let mut n = if m < l && m > -l { l } else { 0 };
                    while n <= l {
                        block_pos_mutable.0.x = block_pos.0.x + m;
                        block_pos_mutable.0.y = block_pos.0.y + k - 1;
                        block_pos_mutable.0.z = block_pos.0.z + n;
                        // Make sure the world lock is dropped
                        {
                            let world = &mob.get_entity().world;
                            let can_target = if let Some(x) = self.move_to_target_pos.upgrade() {
                                x.is_target_pos(world.clone(), block_pos_mutable).await
                            } else {
                                false
                            };
                            if mob
                                .get_mob_entity()
                                .is_in_position_target_range_pos(block_pos_mutable)
                                && can_target
                            {
                                self.target_pos.store(block_pos_mutable);
                                return true;
                            }
                        };

                        n = if n > 0 { -n } else { 1 - n };
                    }
                    m = if m > 0 { -m } else { 1 - m };
                }
            }
            k = if k > 0 { -k } else { 1 - k };
        }

        false
    }

    fn get_target_pos(&self) -> BlockPos {
        self.target_pos.load().up()
    }

    fn should_reset_path(&self) -> bool {
        self.trying_time.load(Relaxed) % 40 == 0
    }

    fn start_moving_to_target(_mob: &dyn Mob) {
        // TODO: implement when navigation is implemented
    }
}

// Contains overridable functions
#[async_trait]
pub trait MoveToTargetPos: Send + Sync {
    async fn is_target_pos(&self, world: Arc<World>, block_pos: BlockPos) -> bool;

    fn get_desired_distance_to_target(&self) -> f64 {
        1.0
    }
}

#[async_trait]
impl Goal for MoveToTargetPosGoal {
    async fn can_start(&self, mob: &dyn Mob) -> bool {
        if self.cooldown.load(Relaxed) > 0 {
            self.cooldown.fetch_sub(1, Relaxed);
            return false;
        }
        self.cooldown.store(Self::get_interval(mob), Relaxed);
        self.find_target_pos(mob).await
    }

    async fn should_continue(&self, mob: &dyn Mob) -> bool {
        let world = &mob.get_entity().world;
        let can_target = if let Some(x) = self.move_to_target_pos.upgrade() {
            x.is_target_pos(world.clone(), self.target_pos.load()).await
        } else {
            false
        };
        self.trying_time.load(Relaxed) >= -self.safe_waiting_time.load(Relaxed)
            && self.trying_time.load(Relaxed) <= MAX_TRYING_TIME
            && can_target
    }

    async fn start(&self, mob: &dyn Mob) {
        Self::start_moving_to_target(mob);
        self.trying_time.store(0, Relaxed);
        let random = mob.get_random().random_range(0..MIN_WAITING_TIME);
        self.safe_waiting_time.store(
            mob.get_random().random_range(random..MIN_WAITING_TIME) + MIN_WAITING_TIME,
            Relaxed,
        );
    }

    async fn stop(&self, _mob: &dyn Mob) {}

    async fn tick(&self, mob: &dyn Mob) {
        let block_pos = self.get_target_pos();
        let block_pos: Vector3<f64> = block_pos.0.to_f64();
        let Some(move_to_target_pos) = self.move_to_target_pos.upgrade() else {
            return;
        };
        let desired_distance = move_to_target_pos.get_desired_distance_to_target();
        if block_pos.squared_distance_to_vec(mob.get_entity().pos.load())
            < desired_distance * desired_distance
        {
            self.reached.store(true, Relaxed);
            self.trying_time.fetch_sub(1, Relaxed);
        } else {
            self.reached.store(false, Relaxed);
            self.trying_time.fetch_add(1, Relaxed);
            if self.should_reset_path() {
                // TODO: implement when navigation is implemented
                // this.mob.getNavigation().startMovingTo(lv.getX() + 0.5, lv.getY(), lv.getZ() + 0.5, this.speed);
            }
        }
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn get_goal_control(&self) -> &GoalControl {
        &self.goal_control
    }
}
