use crate::entity::player::Player;
use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

/// An event that occurs when a player damages / breaks a block.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockDamageEvent {
    /// The player damaging the block.
    pub player: Arc<Player>,

    /// The block being damaged.
    pub block: &'static Block,

    /// The position of the block.
    pub block_pos: BlockPos,

    /// Whether the block is instantly broken.
    pub insta_break: bool,
}

impl BlockDamageEvent {
    #[must_use]
    pub const fn new(
        player: Arc<Player>,
        block: &'static Block,
        block_pos: BlockPos,
        insta_break: bool,
    ) -> Self {
        Self {
            player,
            block,
            block_pos,
            insta_break,
            cancelled: false,
        }
    }
}
