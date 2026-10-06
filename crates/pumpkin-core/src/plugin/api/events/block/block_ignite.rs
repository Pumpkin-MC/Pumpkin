use crate::entity::player::Player;
use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

/// An event that occurs when a block ignites.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockIgniteEvent {
    /// The position of the ignited block. The clicked block for campfires, candles and
    /// candle cakes, otherwise the created fire.
    pub block_pos: BlockPos,

    /// The block that ignited this block.
    pub igniting_block: &'static Block,

    /// The player that ignited the block, if any.
    pub player: Option<Arc<Player>>,

    /// What ignited the block, e.g. flint and steel or a fire charge.
    pub cause: String,
}

impl BlockIgniteEvent {
    pub const CAUSE_FLINT_AND_STEEL: &'static str = "flint-and-steel";
    pub const CAUSE_FIRE_CHARGE: &'static str = "fire-charge";

    #[must_use]
    pub const fn new(
        block_pos: BlockPos,
        igniting_block: &'static Block,
        player: Option<Arc<Player>>,
        cause: String,
    ) -> Self {
        Self {
            block_pos,
            igniting_block,
            player,
            cause,
            cancelled: false,
        }
    }
}
