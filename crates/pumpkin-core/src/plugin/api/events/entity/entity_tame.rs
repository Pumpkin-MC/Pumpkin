use crate::entity::player::Player;
use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

/// An event that occurs when an entity is tamed by a player.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityTameEvent {
    /// The ID of the tamed entity.
    pub entity_id: i32,

    /// The player taming the entity.
    pub owner: Arc<Player>,
}

impl EntityTameEvent {
    #[must_use]
    pub const fn new(entity_id: i32, owner: Arc<Player>) -> Self {
        Self {
            entity_id,
            owner,
            cancelled: false,
        }
    }
}
