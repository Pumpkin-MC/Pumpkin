use async_trait::async_trait;

use crate::entity::mob::Mob;

pub mod active_target_goal;
pub mod ambient_stand_goal;
pub mod look_around_goal;
pub mod look_at_entity;
pub mod move_to_target_pos_goal;
pub mod step_and_destroy_block_goal;
mod track_target_goal;

#[must_use]
pub fn to_goal_ticks(server_ticks: i32) -> i32 {
    -(-server_ticks).div_euclid(2)
}

#[async_trait]
pub trait Goal: Send + Sync {
    /// How should the `Goal` initially start?
    async fn can_start(&self, mob: &dyn Mob) -> bool;
    /// When it's started, how should it continue to run?
    async fn should_continue(&self, mob: &dyn Mob) -> bool;
    /// Call when goal start
    async fn start(&self, mob: &dyn Mob);
    /// Call when goal stop
    async fn stop(&self, mob: &dyn Mob);
    /// If the `Goal` is running, this gets called every tick.
    async fn tick(&self, mob: &dyn Mob);

    fn should_run_every_tick(&self) -> bool {
        false
    }

    fn get_tick_count(&self, ticks: i32) -> i32 {
        if self.should_run_every_tick() {
            ticks
        } else {
            to_goal_ticks(ticks)
        }
    }
}
