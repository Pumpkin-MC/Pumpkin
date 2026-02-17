use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;

use super::BlockEvent;

/// An event that occurs when a fluid block's level changes.
#[cancellable]
#[derive(Event, Clone)]
pub struct FluidLevelChangeEvent {
    /// The block whose fluid level is changing.
    pub block: &'static Block,

    /// The block position.
    pub block_pos: BlockPos,

    /// The world UUID where the change occurs.
    pub world_uuid: uuid::Uuid,

    /// The new block state id.
    pub new_state_id: BlockStateId,
}

impl FluidLevelChangeEvent {
    /// Creates a new `FluidLevelChangeEvent`.
    #[must_use]
    pub const fn new(
        block: &'static Block,
        block_pos: BlockPos,
        world_uuid: uuid::Uuid,
        new_state_id: BlockStateId,
    ) -> Self {
        Self {
            block,
            block_pos,
            world_uuid,
            new_state_id,
            cancelled: false,
        }
    }
}

impl BlockEvent for FluidLevelChangeEvent {
    fn get_block(&self) -> &Block {
        self.block
    }
}
