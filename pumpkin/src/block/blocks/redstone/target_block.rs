use async_trait::async_trait;
use pumpkin_data::{Block, BlockState};
use pumpkin_macros::pumpkin_block;

use pumpkin_world::block::BlockDirection;

use crate::block::pumpkin_block::PumpkinBlock;

#[pumpkin_block("minecraft:target")]
pub struct TargetBlock;

#[async_trait]
impl PumpkinBlock for TargetBlock {
    async fn emits_redstone_power(
        &self,
        _block: &Block,
        _state: &BlockState,
        _direction: &BlockDirection,
    ) -> bool {
        true
    }
}
