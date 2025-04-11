use async_trait::async_trait;
use pumpkin_data::{Block, BlockState};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;

use crate::{block::pumpkin_block::PumpkinBlock, world::World};

#[pumpkin_block("minecraft:redstone_block")]
pub struct RedstoneBlock;

#[async_trait]
impl PumpkinBlock for RedstoneBlock {
    async fn get_weak_redstone_power(
        &self,
        _block: &Block,
        _world: &World,
        _block_pos: &BlockPos,
        _state: &BlockState,
        _direction: &BlockDirection,
    ) -> u8 {
        15
    }

    async fn emits_redstone_power(
        &self,
        _block: &Block,
        _state: &BlockState,
        _direction: &BlockDirection,
    ) -> bool {
        true
    }
}
