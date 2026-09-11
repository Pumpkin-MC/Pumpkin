use crate::entity::player::Player;
use pumpkin_macros::Event;
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

/// An event that occurs when a player extracts items from a furnace output slot.
#[derive(Event, Clone)]
pub struct FurnaceExtractEvent {
    /// The player taking the item.
    pub player: Arc<Player>,

    /// The position of the furnace block.
    pub block_pos: BlockPos,

    /// The registry key of the extracted item.
    pub item_id: String,

    /// The amount of item extracted.
    pub item_amount: u32,

    /// The experience gained from extracting.
    pub exp_gained: f32,
}

impl FurnaceExtractEvent {
    #[must_use]
    pub const fn new(
        player: Arc<Player>,
        block_pos: BlockPos,
        item_id: String,
        item_amount: u32,
        exp_gained: f32,
    ) -> Self {
        Self {
            player,
            block_pos,
            item_id,
            item_amount,
            exp_gained,
        }
    }
}
