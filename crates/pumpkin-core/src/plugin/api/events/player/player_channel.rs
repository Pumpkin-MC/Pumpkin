use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use super::PlayerEvent;
use crate::entity::player::Player;

/// An event that occurs when a player changes/registers a plugin messaging channel.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerChannelEvent {
    /// The player involved.
    pub player: Arc<Player>,

    /// The plugin message channel name.
    pub channel: String,
}

impl PlayerEvent for PlayerChannelEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
