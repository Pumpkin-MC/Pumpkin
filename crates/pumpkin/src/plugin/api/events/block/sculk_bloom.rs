use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::world::World;

/// An event that occurs when a sculk catalyst blooms.
#[cancellable]
#[derive(Event, Clone)]
pub struct SculkBloomEvent {
    pub block_pos: BlockPos,
    pub world: Arc<World>,
    pub charge: i32,
}

impl SculkBloomEvent {
    #[must_use]
    pub const fn new(block_pos: BlockPos, world: Arc<World>, charge: i32) -> Self {
        Self {
            block_pos,
            world,
            charge,
            cancelled: false,
        }
    }
}
