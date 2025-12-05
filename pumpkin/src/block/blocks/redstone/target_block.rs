use pumpkin_macros::pumpkin_block;

use crate::block::{BlockBehaviour, BlockFuture, EmitsRedstonePowerArgs};

#[pumpkin_block("minecraft:target")]
pub struct TargetBlock;

impl BlockBehaviour for TargetBlock {
    fn emits_redstone_power<'a>(
        &'a self,
        _args: EmitsRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async move { true })
    }
}
