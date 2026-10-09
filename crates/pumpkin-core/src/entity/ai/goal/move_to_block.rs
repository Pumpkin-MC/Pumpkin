use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use super::{Controls, Goal, to_goal_ticks};
use crate::entity::mob::Mob;
use crate::world::World;

const GIVE_UP_TICKS: i32 = 1200;
const STAY_TICKS: i32 = 1200;
const INTERVAL_TICKS: i32 = 200;

pub type BlockTargetPredicate = Box<dyn Fn(&World, BlockPos) -> bool + Send + Sync>;

/// Generic goal for navigating towards a target block in the world.
/// Mirrors vanilla Minecraft's `net.minecraft.world.entity.ai.goal.MoveToBlockGoal`.
pub struct MoveToBlockGoal {
    goal_control: Controls,
    pub speed: f64,
    pub search_range: i32,
    pub vertical_search_range: i32,
    pub vertical_search_start: i32,
    pub target_pos: BlockPos,
    pub reached_target: bool,
    pub try_ticks: i32,
    pub max_stay_ticks: i32,
    pub next_start_tick: i32,
    pub accepted_distance: f64,
    pub recalculate_interval: i32,
    pub is_valid_target: BlockTargetPredicate,
}

impl MoveToBlockGoal {
    #[must_use]
    pub fn new(
        speed: f64,
        search_range: i32,
        vertical_search_range: i32,
        is_valid_target: BlockTargetPredicate,
    ) -> Self {
        Self {
            goal_control: Controls::MOVE | Controls::JUMP,
            speed,
            search_range,
            vertical_search_range,
            vertical_search_start: 0,
            target_pos: BlockPos::ZERO,
            reached_target: false,
            try_ticks: 0,
            max_stay_ticks: 0,
            next_start_tick: 0,
            accepted_distance: 1.0,
            recalculate_interval: 40,
            is_valid_target,
        }
    }

    #[must_use]
    pub const fn with_accepted_distance(mut self, accepted_distance: f64) -> Self {
        self.accepted_distance = accepted_distance;
        self
    }

    #[must_use]
    pub const fn with_recalculate_interval(mut self, interval: i32) -> Self {
        self.recalculate_interval = interval;
        self
    }

    #[must_use]
    pub fn get_interval(mob: &dyn Mob) -> i32 {
        to_goal_ticks(INTERVAL_TICKS + mob.get_random().random_range(0..INTERVAL_TICKS))
    }

    pub fn find_nearest_block(&mut self, mob: &dyn Mob) -> bool {
        let mob_pos = mob.get_entity().block_pos.load();
        let world = mob.get_entity().world.load();
        let mut check_pos = BlockPos::ZERO;

        let mut y = self.vertical_search_start;
        while y <= self.vertical_search_range {
            for r in 0..self.search_range {
                let mut x = 0;
                while x <= r {
                    let mut z = if x < r && x > -r { r } else { 0 };
                    while z <= r {
                        check_pos.0.x = mob_pos.0.x + x;
                        check_pos.0.y = mob_pos.0.y + y - 1;
                        check_pos.0.z = mob_pos.0.z + z;

                        if (self.is_valid_target)(&world, check_pos) {
                            self.target_pos = check_pos;
                            return true;
                        }

                        z = if z > 0 { -z } else { 1 - z };
                    }
                    x = if x > 0 { -x } else { 1 - x };
                }
            }
            y = if y > 0 { -y } else { 1 - y };
        }
        false
    }

    #[must_use]
    pub fn get_move_to_target(&self) -> BlockPos {
        self.target_pos.up()
    }

    pub fn move_mob_to_block(&self, mob: &dyn Mob) {
        let target = self.get_move_to_target();
        let mob_pos = mob.get_entity().pos.load();
        let dest = Vector3::new(
            f64::from(target.0.x) + 0.5,
            f64::from(target.0.y),
            f64::from(target.0.z) + 0.5,
        );
        mob.navigate_to(mob_pos, dest, self.speed);
    }

    #[must_use]
    pub const fn is_reached_target(&self) -> bool {
        self.reached_target
    }
}

impl Goal for MoveToBlockGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if self.next_start_tick > 0 {
            self.next_start_tick -= 1;
            return false;
        }
        self.next_start_tick = Self::get_interval(mob);
        self.find_nearest_block(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let world = mob.get_entity().world.load();
        let valid = (self.is_valid_target)(&world, self.target_pos);
        self.try_ticks >= -self.max_stay_ticks && self.try_ticks <= GIVE_UP_TICKS && valid
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.move_mob_to_block(mob);
        self.try_ticks = 0;
        let bound = mob.get_random().random_range(0..STAY_TICKS) + STAY_TICKS;
        self.max_stay_ticks = mob.get_random().random_range(0..bound) + STAY_TICKS;
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let target = self.get_move_to_target();
        let target_center = Vector3::new(
            f64::from(target.0.x) + 0.5,
            f64::from(target.0.y) + 0.5,
            f64::from(target.0.z) + 0.5,
        );
        let mob_pos = mob.get_entity().pos.load();

        if target_center.squared_distance_to_vec(&mob_pos)
            < self.accepted_distance * self.accepted_distance
        {
            self.reached_target = true;
            self.try_ticks -= 1;
        } else {
            self.reached_target = false;
            self.try_ticks += 1;
            if self.try_ticks % self.recalculate_interval == 0 || mob.is_navigator_idle() {
                self.move_mob_to_block(mob);
            }
        }
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.target_pos = BlockPos::ZERO;
        self.reached_target = false;
        mob.stop_navigation();
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
