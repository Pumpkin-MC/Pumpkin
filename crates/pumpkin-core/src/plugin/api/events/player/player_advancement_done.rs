use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use super::PlayerEvent;
use crate::entity::player::Player;

/// An event that occurs when a player completes an advancement.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerAdvancementDoneEvent {
    /// The player who completed the advancement.
    pub player: Arc<Player>,

    /// The advancement identifier.
    pub advancement_id: String,
}

impl PlayerAdvancementDoneEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, advancement_id: String) -> Self {
        Self {
            player,
            advancement_id,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerAdvancementDoneEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
