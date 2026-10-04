use pumpkin_data::BlockDirection;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::entity::{EntityBase, player::Player};

/// An event that occurs when a hanging entity is placed.
#[cancellable]
#[derive(Event, Clone)]
pub struct HangingPlaceEvent {
    /// The hanging entity being placed.
    pub entity: Arc<dyn EntityBase>,
    /// The player placing the entity, if placed by a player.
    pub player: Option<Arc<Player>>,
    /// The position of the block against which the entity is placed.
    pub block_pos: BlockPos,
    /// The face of the block where the entity is placed.
    pub block_face: BlockDirection,
}

impl HangingPlaceEvent {
    #[must_use]
    pub const fn new(
        entity: Arc<dyn EntityBase>,
        player: Option<Arc<Player>>,
        block_pos: BlockPos,
        block_face: BlockDirection,
    ) -> Self {
        Self {
            entity,
            player,
            block_pos,
            block_face,
            cancelled: false,
        }
    }
}
