use pumpkin_macros::pumpkin_block;

use crate::block::BlockBehaviour;

#[pumpkin_block("minecraft:structure_void")]
pub struct StructureVoidBlock;

impl BlockBehaviour for StructureVoidBlock {}
