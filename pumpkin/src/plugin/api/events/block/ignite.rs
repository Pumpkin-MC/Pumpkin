use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::entity::player::Player;

use super::BlockEvent;

/// An event that occurs when a block is ignited.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockIgniteEvent {
    /// The player who ignited the block.
    pub player: Arc<Player>,

    /// The block that was ignited.
    pub block: &'static Block,

    /// The block that caused the ignition.
    pub igniting_block: &'static Block,

    /// The block position that is ignited.
    pub block_pos: BlockPos,

    /// The world UUID where the block ignited.
    pub world_uuid: uuid::Uuid,

    /// The ignition cause.
    pub cause: String,
}

impl BlockEvent for BlockIgniteEvent {
    fn get_block(&self) -> &Block {
        self.block
    }
}
