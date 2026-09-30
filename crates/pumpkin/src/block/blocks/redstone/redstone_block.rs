use pumpkin_macros::pumpkin_block;

use crate::block::{BlockBehaviour, EmitsRedstonePowerArgs, GetRedstonePowerArgs};

#[pumpkin_block("minecraft:redstone_block")]
pub struct RedstoneBlock;

impl BlockBehaviour for RedstoneBlock {
    fn get_weak_redstone_power(&self, _args: GetRedstonePowerArgs<'_>) -> u8 {
        15
    }

    fn emits_redstone_power(&self, _args: EmitsRedstonePowerArgs<'_>) -> bool {
        true
    }
}
