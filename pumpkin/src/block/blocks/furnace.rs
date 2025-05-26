use async_trait::async_trait;
use pumpkin_macros::pumpkin_block;

use crate::block::pumpkin_block::PumpkinBlock;

#[pumpkin_block("minecraft:furnace")]
pub struct FurnaceBlock;

#[async_trait]
impl PumpkinBlock for FurnaceBlock {}
