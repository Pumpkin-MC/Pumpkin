use async_trait::async_trait;

use crate::entity::mob::MobEntity;

pub mod look_at_entity;
pub mod target_goal;

#[async_trait]
pub trait Goal: Send + Sync {
    /// How should the `Goal` initially start?
    async fn can_start(&self, mob: &MobEntity) -> bool;
    /// When it's started, how should it continue to run?
    async fn should_continue(&self, mob: &MobEntity) -> bool;
    /// Call when goal start
    async fn start(&self, mob: &MobEntity);
    /// Call when goal stop
    async fn stop(&self, mob: &MobEntity);
    /// If the `Goal` is running, this gets called every tick.
    async fn tick(&self, mob: &MobEntity);

    fn should_run_every_tick(&self) -> bool {
        false
    }

    fn get_tick_count(&self, ticks: i32) -> i32 {
        if self.should_run_every_tick() { ticks } else { self.to_goal_ticks(ticks) }
    }

    fn to_goal_ticks(&self, server_ticks: i32) -> i32 {
        -(-server_ticks).div_euclid(2)
    }
}
