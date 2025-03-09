use pumpkin_macros::{Event, cancellable};
use pumpkin_util::text::TextComponent;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs before a player joins the game.
///
/// If the event is cancelled, the player will be kicked from the server.
///
/// This event contains information about the player joining and has the option to set a kick message.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerLoginEvent {
    /// The player who is joining the game.
    pub player: Arc<Player>,

    /// The kick message to display if the event is cancelled.
    pub kick_message: TextComponent,
}

impl PlayerLoginEvent {
    /// Creates a new instance of `PlayerLoginEvent`.
    ///
    /// # Arguments
    /// - `player`: A reference to the player joining the game.
    /// - `kick_message`: The message to display upon joining.
    ///
    /// # Returns
    /// A new instance of `PlayerLoginEvent`.
    pub fn new(player: Arc<Player>, kick_message: TextComponent) -> Self {
        Self {
            player,
            kick_message,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerLoginEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
