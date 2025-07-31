use async_trait::async_trait;
use pumpkin_macros::pumpkin_block;

use crate::block::{BlockBehaviour, EmitsRedstonePowerArgs};

#[pumpkin_block("minecraft:target")]
pub struct TargetBlock;

#[async_trait]
impl BlockBehaviour for TargetBlock {
    async fn emits_redstone_power(&self, _args: EmitsRedstonePowerArgs<'_>) -> bool {
        true
    }
}
