use pumpkin_data::BlockStateId;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::{entity::EntityBase, world::World};

/// An event that occurs when a block forms as a result of an entity's actions (e.g. frost walker, snow golem).
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityBlockFormEvent {
    pub entity: Arc<dyn EntityBase>,
    pub block_pos: BlockPos,
    pub world: Arc<World>,
    pub new_state_id: BlockStateId,
}

impl EntityBlockFormEvent {
    #[must_use]
    pub const fn new(
        entity: Arc<dyn EntityBase>,
        block_pos: BlockPos,
        world: Arc<World>,
        new_state_id: BlockStateId,
    ) -> Self {
        Self {
            entity,
            block_pos,
            world,
            new_state_id,
            cancelled: false,
        }
    }
}
