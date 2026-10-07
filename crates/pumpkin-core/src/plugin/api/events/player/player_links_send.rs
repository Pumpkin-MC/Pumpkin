use super::PlayerEvent;
use crate::entity::player::Player;
use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

/// An event that occurs when server links are sent to a player.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerLinksSendEvent {
    /// The player.
    pub player: Arc<Player>,
    /// Links sent.
    pub links: Vec<String>,
}

impl PlayerLinksSendEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, links: Vec<String>) -> Self {
        Self {
            player,
            links,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerLinksSendEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
