use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player teleports.
///
/// If the event is cancelled, the teleportation will not happen.
///
/// This event contains information about the player, the position from which the player teleported, and the position to which the player teleported.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerTeleportEvent {
    /// The player who teleported.
    pub player: Arc<Player>,

    /// The position from which the player teleported.
    pub from: Vector3<f64>,

    /// The position to which the player teleported.
    pub to: Vector3<f64>,
}

impl PlayerTeleportEvent {
    /// Creates a new instance of `PlayerTeleportEvent`.
    ///
    /// # Arguments
    /// - `player`: A reference to the player who teleported.
    /// - `from`: The position from which the player teleported.
    /// - `to`: The position to which the player teleported.
    ///
    /// # Returns
    /// A new instance of `PlayerTeleportEvent`.
    pub fn new(player: Arc<Player>, from: Vector3<f64>, to: Vector3<f64>) -> Self {
        Self {
            player,
            from,
            to,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerTeleportEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
