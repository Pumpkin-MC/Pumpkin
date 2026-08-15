use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::world::World;

/// An event that occurs when a bell resonates and highlights nearby raiders.
#[cancellable]
#[derive(Event, Clone)]
pub struct BellResonateEvent {
    pub block_pos: BlockPos,
    pub world: Arc<World>,
}

impl BellResonateEvent {
    #[must_use]
    pub const fn new(block_pos: BlockPos, world: Arc<World>) -> Self {
        Self {
            block_pos,
            world,
            cancelled: false,
        }
    }
}
