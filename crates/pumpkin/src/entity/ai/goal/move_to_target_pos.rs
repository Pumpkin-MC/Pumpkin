use super::{Controls, Goal, to_goal_ticks};

use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::{ai::goal::ParentHandle, mob::Mob};
use crate::world::World;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;
use std::sync::Arc;

const GIVE_UP_TICKS: i32 = 1200;
const STAY_TICKS: i32 = 1200;
const INTERVAL_TICKS: i32 = 200;

/// Mirrors vanilla Minecraft's `net.minecraft.world.entity.ai.goal.MoveToBlockGoal`.
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

    #[must_use]
    pub fn with_default(move_to_target_pos: ParentHandle<M>, speed: f64, range: i32) -> Self {
        Self::new(move_to_target_pos, speed, range, 1)
    }

    pub fn get_interval(mob: &dyn Mob) -> i32 {
        to_goal_ticks(INTERVAL_TICKS + mob.get_random().random_range(0..INTERVAL_TICKS))
    }

    pub fn find_target_pos(&mut self, mob: &dyn Mob) -> bool {
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
                            let world = mob.get_entity().world.load_full();

                            let can_target =
                                self.move_to_target_pos
                                    .get()
                                    .is_some_and(|move_to_target_pos| {
                                        move_to_target_pos.is_target_pos(world, block_pos_mut)
                                    });

                            if mob
                                .get_mob_entity()
                                .is_in_position_target_range_pos(&block_pos_mut)
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

    #[must_use]
    pub fn get_target_pos(&self) -> BlockPos {
        self.target_pos.up()
    }

    #[must_use]
    pub const fn should_reset_path(&self) -> bool {
        self.trying_time % 40 == 0
    }

    #[must_use]
    pub const fn is_reached_target(&self) -> bool {
        self.reached
    }

    pub fn move_mob_to_block(&self, mob: &dyn Mob) {
        let target = self.get_target_pos();
        let mut navigator = mob
            .get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        navigator.set_progress(NavigatorGoal {
            current_progress: mob.get_entity().pos.load(),
            destination: Vector3::new(
                target.0.x as f64 + 0.5,
                target.0.y as f64,
                target.0.z as f64 + 0.5,
            ),
            speed: self.speed,
        });
    }
}

// Contains overridable functions
pub trait MoveToTargetPos: Send + Sync {
    fn is_target_pos(&self, world: Arc<World>, block_pos: BlockPos) -> bool;

    fn get_desired_distance_to_target(&self) -> f64 {
        1.0
    }
}

impl<M: MoveToTargetPos> Goal for MoveToTargetPosGoal<M> {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if self.cooldown > 0 {
            self.cooldown -= 1;
            return false;
        }
        self.cooldown = Self::get_interval(mob);
        self.find_target_pos(mob)
    }

    fn should_continue(&self, mob: &dyn Mob) -> bool {
        let world = mob.get_entity().world.load_full();
        let can_target = self
            .move_to_target_pos
            .get()
            .is_some_and(|move_to_target_pos| {
                move_to_target_pos.is_target_pos(world, self.target_pos)
            });
        self.trying_time >= -self.safe_waiting_time
            && self.trying_time <= GIVE_UP_TICKS
            && can_target
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.move_mob_to_block(mob);
        self.trying_time = 0;
        let bound = mob.get_random().random_range(0..STAY_TICKS) + STAY_TICKS;
        self.safe_waiting_time = mob.get_random().random_range(0..bound) + STAY_TICKS;
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let target = self.get_target_pos();
        let target_center = Vector3::new(
            target.0.x as f64 + 0.5,
            target.0.y as f64 + 0.5,
            target.0.z as f64 + 0.5,
        );
        let Some(move_to_target_pos) = self.move_to_target_pos.get() else {
            return;
        };
        let desired_distance = move_to_target_pos.get_desired_distance_to_target();

        if target_center.squared_distance_to_vec(&mob.get_entity().pos.load())
            < desired_distance * desired_distance
        {
            self.reached = true;
            self.trying_time -= 1;
        } else {
            self.reached = false;
            self.trying_time += 1;
            if self.should_reset_path() {
                let mut navigator = mob
                    .get_mob_entity()
                    .navigator
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);

                navigator.set_progress(NavigatorGoal {
                    current_progress: mob.get_entity().pos.load(),
                    destination: Vector3::new(
                        target.0.x as f64 + 0.5,
                        target.0.y as f64,
                        target.0.z as f64 + 0.5,
                    ),
                    speed: self.speed,
                });
            }
        }
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
