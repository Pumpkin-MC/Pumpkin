use pumpkin_macros::{Event, cancellable};
use pumpkin_util::GameMode;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player changes gamemode.
///
/// This event contains information about
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerGamemodeChangeEvent {
    /// The player who's gamemode is changing.
    pub player: Arc<Player>,

    /// Previous gamemode of the player.
    pub previous_gamemode: GameMode,

    /// New gamemode of the player.
    pub new_gamemode: GameMode,
}

impl PlayerGamemodeChangeEvent {
    /// Creates a new instance of `PlayerGamemodeChangeEvent`.
    ///
    /// # Arguments
    /// - `player`: A reference to the player who is changing gamemode.
    /// - `previous_gamemode`: The previous gamemode of the player.
    /// - `new_gamemode`: The new gamemode of the player.
    ///
    /// # Returns
    /// A new instance of `PlayerGamemodeChangeEvent`.
    pub fn new(player: Arc<Player>, previous_gamemode: GameMode, new_gamemode: GameMode) -> Self {
        Self {
            player,
            previous_gamemode,
            new_gamemode,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerGamemodeChangeEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
