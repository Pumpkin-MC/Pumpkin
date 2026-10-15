use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::{entity::EntityBase, world::World};

/// An event that occurs when a block receives a game event (e.g. sculk sensor).
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockReceiveGameEvent {
    pub block_pos: BlockPos,
    pub world: Arc<World>,
    pub game_event: String,
    pub source_entity: Option<Arc<dyn EntityBase>>,
}

impl BlockReceiveGameEvent {
    #[must_use]
    pub const fn new(
        block_pos: BlockPos,
        world: Arc<World>,
        game_event: String,
        source_entity: Option<Arc<dyn EntityBase>>,
    ) -> Self {
        Self {
            block_pos,
            world,
            game_event,
            source_entity,
            cancelled: false,
        }
    }
}
