use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player toggles their sneaking state.
///
/// If the event is cancelled, the sneaking state will not change.
///
/// This event contains information about the player and whether they are now sneaking.
///
/// # Note
///
/// Cancelling this event will only prevent the server from processing the state change.  
/// The player’s client will still reflect the attempted action, meaning that the sneaking animation may still  
/// display on the initiating client, even though other players won't see the sneak action.
///
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerToggleSneakEvent {
    /// The player who is toggling their sneaking state.
    pub player: Arc<Player>,

    /// Whether the player is now sneaking (true) or not (false).
    pub is_sneaking: bool,
}

impl PlayerToggleSneakEvent {
    /// Creates a new instance of `PlayerToggleSneakEvent`.
    ///
    /// # Arguments
    /// - `player`: A reference to the player who is toggling their sneaking state.
    /// - `is_sneaking`: Whether the player is now sneaking (true) or not (false).
    ///
    /// # Returns
    /// A new instance of `PlayerToggleSneakEvent`.
    ///
    #[must_use]
    pub fn new(player: Arc<Player>, is_sneaking: bool) -> Self {
        Self {
            player,
            is_sneaking,
            cancelled: false,
        }
    }

    #[must_use]
    pub fn is_sneaking(&self) -> bool {
        self.is_sneaking
    }
}

impl PlayerEvent for PlayerToggleSneakEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
