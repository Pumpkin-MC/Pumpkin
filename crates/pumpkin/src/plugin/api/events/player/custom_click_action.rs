use bytes::Bytes;
use pumpkin_macros::Event;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player clicks a custom dialog button.
#[derive(Event, Clone)]
pub struct CustomClickActionEvent {
    /// The player who clicked the button.
    pub player: Arc<Player>,
    /// The unique identifier for the action.
    pub id: String,
    /// Optional binary data associated with the action.
    pub payload: Option<Bytes>,
}

impl CustomClickActionEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, id: String, payload: Option<Bytes>) -> Self {
        Self {
            player,
            id,
            payload,
        }
    }
}

impl PlayerEvent for CustomClickActionEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
