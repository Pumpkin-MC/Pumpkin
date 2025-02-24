use pumpkin_macros::{Event, cancellable};
use pumpkin_util::text::TextComponent;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player joins the game.
///
/// This event contains information about the player joining and a message to display upon joining.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerJoinEvent {
    /// The player who is joining the game.
    pub player: Arc<Player>,

    /// The message to display when the player joins.
    pub join_message: TextComponent,
}

impl PlayerJoinEvent {
    /// Creates a new instance of `PlayerJoinEvent`.
    ///
    /// # Arguments
    /// - `player`: A reference to the player joining the game.
    /// - `join_message`: The message to display upon joining.
    ///
    /// # Returns
    /// A new instance of `PlayerJoinEvent`.
    pub fn new(player: Arc<Player>, join_message: TextComponent) -> Self {
        Self {
            player,
            join_message,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerJoinEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
