use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player sends a chat message.
///
/// This event contains information about the sender, message, and recipients.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerChatEvent {
    /// The player who sent the message.
    pub player: Arc<Player>,

    /// The message being sent.
    pub message: String,

    /// The recipients of the message. If empty, the message is broadcasted to all players.
    pub recipients: Vec<Arc<Player>>,
}

impl PlayerChatEvent {
    /// Creates a new instance of `PlayerChatEvent`.
    ///
    /// # Arguments
    /// - `player`: A reference to the player sending the message.
    /// - `message`: The message being sent.
    /// - `recipients`: The recipients of the message. If empty, the message is broadcasted to all players.
    ///
    /// # Returns
    /// A new instance of `PlayerChatEvent`.
    pub fn new(player: Arc<Player>, message: String, recipients: Vec<Arc<Player>>) -> Self {
        Self {
            player,
            message,
            recipients,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerChatEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
