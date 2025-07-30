use async_trait::async_trait;
use pumpkin_macros::pumpkin_block;

use crate::block::{BlockBehaviour, EmitsRedstonePowerArgs, GetRedstonePowerArgs};

#[pumpkin_block("minecraft:redstone_block")]
pub struct RedstoneBlock;

#[async_trait]
impl BlockBehaviour for RedstoneBlock {
    async fn get_weak_redstone_power(&self, _args: GetRedstonePowerArgs<'_>) -> u8 {
        15
    }

    async fn emits_redstone_power(&self, _args: EmitsRedstonePowerArgs<'_>) -> bool {
        true
    }
}
