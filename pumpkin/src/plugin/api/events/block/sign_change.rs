use pumpkin_data::Block;
use pumpkin_macros::{cancellable, Event};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::entity::player::Player;

use super::BlockEvent;

/// An event that occurs when a sign's text changes.
#[cancellable]
#[derive(Event, Clone)]
pub struct SignChangeEvent {
    /// The player who edited the sign.
    pub player: Arc<Player>,

    /// The sign block.
    pub block: &'static Block,

    /// The sign position.
    pub block_pos: BlockPos,

    /// The sign lines (length 4).
    pub lines: Vec<String>,

    /// Whether the front text was edited.
    pub is_front_text: bool,
}

impl SignChangeEvent {
    /// Creates a new `SignChangeEvent`.
    #[must_use]
    pub const fn new(
        player: Arc<Player>,
        block: &'static Block,
        block_pos: BlockPos,
        lines: Vec<String>,
        is_front_text: bool,
    ) -> Self {
        Self {
            player,
            block,
            block_pos,
            lines,
            is_front_text,
            cancelled: false,
        }
    }
}

impl BlockEvent for SignChangeEvent {
    fn get_block(&self) -> &Block {
        self.block
    }
}
