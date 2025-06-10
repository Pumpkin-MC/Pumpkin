use crate::block::pumpkin_block::PumpkinBlock;
use async_trait::async_trait;
use pumpkin_macros::pumpkin_block;

#[pumpkin_block("minecraft:crafting_table")]
pub struct CraftingTableBlock;

#[async_trait]
impl PumpkinBlock for CraftingTableBlock {}
