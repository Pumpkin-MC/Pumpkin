use async_trait::async_trait;
use pumpkin_macros::pumpkin_block;

use crate::block::pumpkin_block::{EmitsRedstonePowerArgs, PumpkinBlock};

#[pumpkin_block("minecraft:target")]
pub struct TargetBlock;

#[async_trait]
impl PumpkinBlock for TargetBlock {
    async fn emits_redstone_power(&self, _args: EmitsRedstonePowerArgs<'_>) -> bool {
        true
    }
}
