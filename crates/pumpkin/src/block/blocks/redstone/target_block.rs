use pumpkin_macros::pumpkin_block;

use crate::block::{BlockBehaviour, EmitsRedstonePowerArgs};

#[pumpkin_block("minecraft:target")]
pub struct TargetBlock;

impl BlockBehaviour for TargetBlock {
    fn emits_redstone_power(&self, _args: EmitsRedstonePowerArgs<'_>) -> bool {
        true
    }
}
