use super::PlayerEvent;
use crate::entity::player::Player;
use pumpkin_macros::Event;
use std::sync::Arc;

/// An event that occurs when a player respawns after dying.
///
/// This event is triggered after a player has died and chosen to respawn, containing
/// information about the player who respawned.
#[derive(Event, Clone)]
pub struct PlayerRespawnEvent {
    /// The player who is respawning.
    pub player: Arc<Player>,
}

impl PlayerRespawnEvent {
    /// Creates a new instance of `PlayerRespawnEvent`.
    ///
    /// # Arguments
    /// - `player`: A reference to the player who is respawning.
    ///
    /// # Returns
    /// A new instance of `PlayerRespawnEvent`.
    pub const fn new(player: Arc<Player>) -> Self {
        Self { player }
    }
}

impl PlayerEvent for PlayerRespawnEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
