//! Fletching table — vanilla job site for fletcher villagers.
//!
//! **Vanilla has no player GUI** for this block (unlike cartography / smithing).
//! Right-click is a no-op success so the client does not treat it as a failed
//! interaction.

use crate::block::registry::BlockActionResult;
use crate::block::{BlockBehaviour, BlockFuture, NormalUseArgs};
use pumpkin_macros::pumpkin_block;

#[pumpkin_block("minecraft:fletching_table")]
pub struct FletchingTableBlock;

impl BlockBehaviour for FletchingTableBlock {
    fn normal_use<'a>(&'a self, _args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        // Vanilla: no container. Do not open a screen.
        Box::pin(async move { BlockActionResult::Success })
    }
}
