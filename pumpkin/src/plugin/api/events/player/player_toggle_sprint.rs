use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player toggles their sprinting state.
///
/// If the event is cancelled, the sprinting state will not change.
///
/// This event contains information about the player and whether they are now sprinting.
///
/// # Note
///
/// Cancelling this event will only prevent the server from processing the state change.  
/// The player’s client will still reflect the attempted action, meaning that the sprinting animation may still  
/// display on the initiating client, even though other players won't see the sprinting action.
///
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerToggleSprintEvent {
    /// The player who is toggling their sprinting state.
    pub player: Arc<Player>,

    /// Whether the player is now sprinting (true) or not (false).
    pub is_sprinting: bool,
}

impl PlayerToggleSprintEvent {
    /// Creates a new instance of `PlayerToggleSprintEvent`.
    ///
    /// # Arguments
    /// - `player`: A reference to the player who is toggling their sprinting state.
    /// - `is_sprinting`: Whether the player is now sprinting (true) or not (false).
    ///
    /// # Returns
    /// A new instance of `PlayerToggleSprintEvent`.
    #[must_use]
    pub fn new(player: Arc<Player>, is_sprinting: bool) -> Self {
        Self {
            player,
            is_sprinting,
            cancelled: false,
        }
    }

    #[must_use]
    pub fn is_sprinting(&self) -> bool {
        self.is_sprinting
    }
}

impl PlayerEvent for PlayerToggleSprintEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
