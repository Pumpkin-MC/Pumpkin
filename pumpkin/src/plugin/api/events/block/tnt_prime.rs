use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::entity::player::Player;

use super::BlockEvent;

/// An event that occurs when TNT is primed.
#[cancellable]
#[derive(Event, Clone)]
pub struct TNTPrimeEvent {
    /// The player that primed the TNT, if any.
    pub player: Option<Arc<Player>>,

    /// The TNT block.
    pub block: &'static Block,

    /// The TNT position.
    pub block_pos: BlockPos,

    /// The world UUID where the TNT is primed.
    pub world_uuid: uuid::Uuid,

    /// The priming cause.
    pub cause: String,
}

impl TNTPrimeEvent {
    /// Creates a new `TNTPrimeEvent`.
    #[must_use]
    pub const fn new(
        player: Option<Arc<Player>>,
        block: &'static Block,
        block_pos: BlockPos,
        world_uuid: uuid::Uuid,
        cause: String,
    ) -> Self {
        Self {
            player,
            block,
            block_pos,
            world_uuid,
            cause,
            cancelled: false,
        }
    }
}

impl BlockEvent for TNTPrimeEvent {
    fn get_block(&self) -> &Block {
        self.block
    }
}
