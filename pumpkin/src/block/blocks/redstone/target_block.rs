use async_trait::async_trait;
use pumpkin_data::{Block, BlockDirection, BlockState};
use pumpkin_macros::pumpkin_block;

use crate::block::pumpkin_block::PumpkinBlock;

#[pumpkin_block("minecraft:target")]
pub struct TargetBlock;

#[async_trait]
impl PumpkinBlock for TargetBlock {
    async fn emits_redstone_power(
        &self,
        _block: &Block,
        _state: &BlockState,
        _direction: BlockDirection,
    ) -> bool {
        true
    }
}
