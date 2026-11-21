use pumpkin_data::BlockStateId;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::{entity::player::Player, world::World};

/// An event that occurs when multiple blocks are placed at once (e.g. bed, door).
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockMultiPlaceEvent {
    pub player: Arc<Player>,
    pub world: Arc<World>,
    pub placed_blocks: Vec<(BlockPos, BlockStateId)>,
}

impl BlockMultiPlaceEvent {
    #[must_use]
    pub const fn new(
        player: Arc<Player>,
        world: Arc<World>,
        placed_blocks: Vec<(BlockPos, BlockStateId)>,
    ) -> Self {
        Self {
            player,
            world,
            placed_blocks,
            cancelled: false,
        }
    }
}
