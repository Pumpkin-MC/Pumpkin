use async_trait::async_trait;
use pumpkin_macros::pumpkin_block;

use crate::block::pumpkin_block::PumpkinBlock;

#[pumpkin_block("minecraft:piston_head")]
pub struct PistonHeadBlock;

#[async_trait]
impl PumpkinBlock for PistonHeadBlock {}
