use std::sync::Arc;

use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;

use crate::world::World;

use super::BlockEvent;

/// An event that occurs when a block grows.
///
/// Scope:
/// - Fired for crop random-tick growth.
/// - Not fired yet for bonemeal growth, sapling/tree growth, kelp/cactus/sugar cane growth,
///   mushroom spread, or other non-crop growth paths.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockGrowEvent {
    /// The world where growth is happening.
    pub world: Arc<World>,

    /// The original block before growth.
    pub old_block: &'static Block,

    /// The original block state id.
    pub old_state_id: BlockStateId,

    /// The new block targeted by growth.
    pub new_block: &'static Block,

    /// The new block state id to apply.
    pub new_state_id: BlockStateId,

    /// The position of the growing block.
    pub block_pos: BlockPos,
}

impl BlockGrowEvent {
    /// Creates a new `BlockGrowEvent`.
    ///
    /// # Arguments
    /// - `world`: The world where the growth is happening.
    /// - `old_block`: The original block before growth.
    /// - `old_state_id`: The original block state id.
    /// - `new_block`: The new block targeted by the growth.
    /// - `new_state_id`: The new block state id that will be applied if not cancelled.
    /// - `block_pos`: The block position where growth is happening.
    ///
    /// # Returns
    /// A new `BlockGrowEvent`.
    #[must_use]
    pub const fn new(
        world: Arc<World>,
        old_block: &'static Block,
        old_state_id: BlockStateId,
        new_block: &'static Block,
        new_state_id: BlockStateId,
        block_pos: BlockPos,
    ) -> Self {
        Self {
            world,
            old_block,
            old_state_id,
            new_block,
            new_state_id,
            block_pos,
            cancelled: false,
        }
    }
}

impl BlockEvent for BlockGrowEvent {
    fn get_block(&self) -> &Block {
        self.old_block
    }
}
