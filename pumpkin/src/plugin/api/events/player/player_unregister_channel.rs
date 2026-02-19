use pumpkin_macros::Event;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player unregisters a plugin channel.
#[derive(Event, Clone)]
pub struct PlayerUnregisterChannelEvent {
    /// The player unregistering the channel.
    pub player: Arc<Player>,

    /// The channel being unregistered.
    pub channel: String,
}

impl PlayerUnregisterChannelEvent {
    /// Creates a new instance of `PlayerUnregisterChannelEvent`.
    #[expect(clippy::missing_const_for_fn)]
    pub fn new(player: Arc<Player>, channel: String) -> Self {
        Self { player, channel }
    }
}

impl PlayerEvent for PlayerUnregisterChannelEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
