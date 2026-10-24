use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use super::PlayerEvent;
use crate::entity::player::Player;

/// An event that occurs when an entity is made visible to a player.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerShowEntityEvent {
    /// The observing player.
    pub player: Arc<Player>,

    /// The entity ID revealed.
    pub entity_id: i32,
}

impl PlayerEvent for PlayerShowEntityEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
