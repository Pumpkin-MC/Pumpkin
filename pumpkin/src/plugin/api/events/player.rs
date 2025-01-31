use pumpkin_macros::{cancellable, Event};
use pumpkin_util::text::TextComponent;
use std::sync::Arc;

use crate::entity::player::Player;

/// A trait representing events related to players.
///
/// This trait provides a method to retrieve the player associated with the event.
pub trait PlayerEvent: Send + Sync {
    /// Retrieves a reference to the player associated with the event.
    ///
    /// # Returns
    /// A reference to the `Arc<Player>` involved in the event.
    fn get_player(&self) -> &Arc<Player>;
}

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

/// An event that occurs when a player leaves the game.
///
/// This event contains information about the player leaving and a message to display upon leaving.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerLeaveEvent {
    /// The player who is leaving the game.
    pub player: Arc<Player>,

    /// The message to display when the player leaves.
    pub leave_message: TextComponent,
}

impl PlayerLeaveEvent {
    /// Creates a new instance of `PlayerLeaveEvent`.
    ///
    /// # Arguments
    /// - `player`: A reference to the player leaving the game.
    /// - `leave_message`: The message to display upon leaving.
    ///
    /// # Returns
    /// A new instance of `PlayerLeaveEvent`.
    pub fn new(player: Arc<Player>, leave_message: TextComponent) -> Self {
        Self {
            player,
            leave_message,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerLeaveEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
