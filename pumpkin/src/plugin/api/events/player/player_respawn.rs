use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player respawns after dying.
///
/// If the event is cancelled, the respawn is prevented.
///
/// This event contains information about the respawning player
/// and the location where they will respawn.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerRespawnEvent {
    /// The player who is respawning.
    pub player: Arc<Player>,

    /// The position where the player will respawn.
    pub respawn_position: Vector3<f64>,
}

impl PlayerRespawnEvent {
    /// Creates a new instance of `PlayerRespawnEvent`.
    ///
    /// # Arguments
    /// - `player`: A reference to the player who is respawning.
    /// - `respawn_position`: The position where the player will respawn.
    ///
    /// # Returns
    /// A new instance of `PlayerRespawnEvent`.
    pub const fn new(player: Arc<Player>, respawn_position: Vector3<f64>) -> Self {
        Self {
            player,
            respawn_position,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerRespawnEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
