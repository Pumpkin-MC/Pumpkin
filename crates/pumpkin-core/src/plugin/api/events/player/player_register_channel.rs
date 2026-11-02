use super::PlayerEvent;
use crate::entity::player::Player;
use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

/// An event that occurs when a plugin channel is registered for a player.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerRegisterChannelEvent {
    /// The player.
    pub player: Arc<Player>,
    /// Channel name.
    pub channel: String,
}

impl PlayerRegisterChannelEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, channel: String) -> Self {
        Self {
            player,
            channel,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerRegisterChannelEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
