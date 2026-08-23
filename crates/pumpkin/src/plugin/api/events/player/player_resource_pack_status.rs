use super::PlayerEvent;
use crate::entity::player::Player;
use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

/// An event that occurs when a player responds to a resource pack request.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerResourcePackStatusEvent {
    /// The player.
    pub player: Arc<Player>,
    /// Pack ID.
    pub pack_id: String,
    /// Status description.
    pub status: String,
}

impl PlayerResourcePackStatusEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, pack_id: String, status: String) -> Self {
        Self {
            player,
            pack_id,
            status,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerResourcePackStatusEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
