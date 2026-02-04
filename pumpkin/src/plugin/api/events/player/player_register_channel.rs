use pumpkin_macros::Event;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player registers a plugin channel.
#[derive(Event, Clone)]
pub struct PlayerRegisterChannelEvent {
    /// The player registering the channel.
    pub player: Arc<Player>,

    /// The channel being registered.
    pub channel: String,
}

impl PlayerRegisterChannelEvent {
    /// Creates a new instance of `PlayerRegisterChannelEvent`.
    pub fn new(player: Arc<Player>, channel: String) -> Self {
        Self { player, channel }
    }
}

impl PlayerEvent for PlayerRegisterChannelEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
