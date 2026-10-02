use crate::entity::player::Player;
use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

/// An event that occurs when a block ignites.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockIgniteEvent {
    /// The position of the ignited block.
    pub block_pos: BlockPos,

    /// The block that ignited this block.
    pub igniting_block: &'static Block,

    /// The player that ignited the block, if any.
    pub player: Option<Arc<Player>>,
}

impl BlockIgniteEvent {
    #[must_use]
    pub const fn new(
        block_pos: BlockPos,
        igniting_block: &'static Block,
        player: Option<Arc<Player>>,
    ) -> Self {
        Self {
            block_pos,
            igniting_block,
            player,
            cancelled: false,
        }
    }
}
