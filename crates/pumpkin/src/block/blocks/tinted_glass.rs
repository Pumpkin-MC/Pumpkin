use pumpkin_macros::pumpkin_block;

use crate::block::BlockBehaviour;

#[pumpkin_block("minecraft:tinted_glass")]
pub struct TintedGlassBlock;

impl BlockBehaviour for TintedGlassBlock {}
