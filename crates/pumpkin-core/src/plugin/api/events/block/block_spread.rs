use pumpkin_data::BlockStateId;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::world::World;

/// An event that occurs when a block spreads to another position based on world conditions (e.g. grass, mycelium, fire).
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockSpreadEvent {
    pub source_pos: BlockPos,
    pub target_pos: BlockPos,
    pub world: Arc<World>,
    pub new_state_id: BlockStateId,
}

impl BlockSpreadEvent {
    #[must_use]
    pub const fn new(
        source_pos: BlockPos,
        target_pos: BlockPos,
        world: Arc<World>,
        new_state_id: BlockStateId,
    ) -> Self {
        Self {
            source_pos,
            target_pos,
            world,
            new_state_id,
            cancelled: false,
        }
    }
}
