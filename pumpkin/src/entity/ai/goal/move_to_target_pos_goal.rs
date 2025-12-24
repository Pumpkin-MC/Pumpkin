use super::{Controls, Goal, to_goal_ticks};
use crate::entity::ai::goal::GoalFuture;
use crate::entity::{ai::goal::ParentHandle, mob::Mob};
use crate::world::World;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::Rng;
use std::pin::Pin;
use std::sync::Arc;

const MIN_WAITING_TIME: i32 = 1200;
const MAX_TRYING_TIME: i32 = 1200;
const MIN_INTERVAL: i32 = 200;

#[allow(dead_code)]
pub struct MoveToTargetPosGoal<M: MoveToTargetPos> {
    goal_control: Controls,
    pub move_to_target_pos: ParentHandle<M>,
    pub speed: f64,
    pub cooldown: i32,
    pub trying_time: i32,
    pub safe_waiting_time: i32,
    pub target_pos: BlockPos,
    pub reached: bool,
    pub range: i32,
    pub max_y_difference: i32,
    pub lowest_y: i32,
}

impl<M: MoveToTargetPos> MoveToTargetPosGoal<M> {
    #[must_use]
    pub fn new(
        move_to_target_pos: ParentHandle<M>,
        speed: f64,
        range: i32,
        max_y_difference: i32,
    ) -> Self {
        Self {
            goal_control: Controls::MOVE | Controls::JUMP,
            move_to_target_pos,
            speed,
            cooldown: 0,
            trying_time: 0,
            safe_waiting_time: 0,
            target_pos: BlockPos::new(0, 0, 0),
            reached: false,
            range,
            max_y_difference,
            lowest_y: 0,
        }
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn with_default(move_to_target_pos: ParentHandle<M>, speed: f64, range: i32) -> Self {
        Self::new(move_to_target_pos, speed, range, 1)
    }

    pub fn get_interval(mob: &dyn Mob) -> i32 {
        to_goal_ticks(MIN_INTERVAL + mob.get_random().random_range(0..MIN_INTERVAL))
    }

    pub async fn find_target_pos(&mut self, mob: &dyn Mob) -> bool {
        let block_pos = mob.get_entity().block_pos.load();
        let mut block_pos_mut = BlockPos::new(0, 0, 0);

        let mut k = self.lowest_y;
        while k <= self.max_y_difference {
            for l in 0..self.range {
                let mut m = 0;
                while m <= l {
                    let mut n = if m < l && m > -l { l } else { 0 };
                    while n <= l {
                        block_pos_mut.0.x = block_pos.0.x + m;
                        block_pos_mut.0.y = block_pos.0.y + k - 1;
                        block_pos_mut.0.z = block_pos.0.z + n;
                        // Make sure the world lock is dropped
                        {
                            let world = &mob.get_entity().world;

                            let can_target =
                                if let Some(move_to_target_pos) = self.move_to_target_pos.get() {
                                    move_to_target_pos
                                        .is_target_pos(world.clone(), block_pos_mut)
                                        .await
                                } else {
                                    false
                                };

                            if mob
                                .get_mob_entity()
                                .is_in_position_target_range_pos(block_pos_mut)
                                && can_target
                            {
                                self.target_pos = block_pos_mut;
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
        self.target_pos.up()
    }

    fn should_reset_path(&self) -> bool {
        self.trying_time % 40 == 0
    }

    fn start_moving_to_target(_mob: &dyn Mob) {
        // TODO: implement when navigation is implemented
    }
}

// Contains overridable functions
pub trait MoveToTargetPos: Send + Sync {
    fn is_target_pos<'a>(
        &'a self,
        world: Arc<World>,
        block_pos: BlockPos,
    ) -> Pin<Box<dyn Future<Output = bool> + Send + 'a>>;

    fn get_desired_distance_to_target(&self) -> f64 {
        1.0
    }
}

impl<M: MoveToTargetPos> Goal for MoveToTargetPosGoal<M> {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            if self.cooldown > 0 {
                self.cooldown -= 1;
                return false;
            }
            self.cooldown = Self::get_interval(mob);
            self.find_target_pos(mob).await
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let world = &mob.get_entity().world;
            let can_target = if let Some(x) = self.move_to_target_pos.get() {
                x.is_target_pos(world.clone(), self.target_pos).await
            } else {
                false
            };
            self.trying_time >= -self.safe_waiting_time
                && self.trying_time <= MAX_TRYING_TIME
                && can_target
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            Self::start_moving_to_target(mob);
            self.trying_time = 0;
            let random = mob.get_random().random_range(0..MIN_WAITING_TIME);
            self.safe_waiting_time =
                mob.get_random().random_range(random..MIN_WAITING_TIME) + MIN_WAITING_TIME;
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            let block_pos = self.get_target_pos();
            let block_pos: Vector3<f64> = block_pos.to_f64();
            let Some(move_to_target_pos) = self.move_to_target_pos.get() else {
                return;
            };
            let desired_distance = move_to_target_pos.get_desired_distance_to_target();

            if block_pos.squared_distance_to_vec(mob.get_entity().pos.load())
                < desired_distance * desired_distance
            {
                self.reached = true;
                self.trying_time -= 1;
            } else {
                self.reached = false;
                self.trying_time += 1;
                if self.should_reset_path() {
                    // TODO: implement when navigation is implemented
                    // this.mob.getNavigation().startMovingTo(lv.getX() + 0.5, lv.getY(), lv.getZ() + 0.5, this.speed);
                }
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
