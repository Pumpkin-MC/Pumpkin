use pumpkin_macros::Event;
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::world::World;

/// An event that occurs when a block gives experience (e.g. mining ore).
#[derive(Event, Clone)]
pub struct BlockExpEvent {
    pub block_pos: BlockPos,
    pub world: Arc<World>,
    pub exp: i32,
}

impl BlockExpEvent {
    #[must_use]
    pub const fn new(block_pos: BlockPos, world: Arc<World>, exp: i32) -> Self {
        Self {
            block_pos,
            world,
            exp,
        }
    }
}
