use async_trait::async_trait;

use crate::entity::mob::MobEntity;

pub mod look_at_entity;

#[async_trait]
pub trait Goal: Send + Sync {
    /// How Should the Goal initially start?
    async fn can_start(&self, mob: &MobEntity) -> bool;
    /// When its started, How it should Continue to run
    async fn should_continue(&self, mob: &MobEntity) -> bool;
    /// If the Goal is running, this gets called every tick
    async fn tick(&self, mob: &MobEntity);
}
