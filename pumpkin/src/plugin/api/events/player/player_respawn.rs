use pumpkin_macros::Event;
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player respawns after death.
///
/// This event cannot be cancelled, but you can modify the respawn position 
/// and other properties through this event.
///
/// This event contains information about the player respawning and their respawn location.
#[derive(Event, Clone)]
pub struct PlayerRespawnEvent {
    /// The player who is respawning.
    pub player: Arc<Player>,

    /// The position where the player will respawn.
    pub respawn_position: Vector3<f64>,

    /// The yaw angle (horizontal rotation) after respawn.
    pub yaw: f32,

    /// The pitch angle (vertical rotation) after respawn.
    pub pitch: f32,
}

impl PlayerRespawnEvent { 
    /// Creates a new instance of `PlayerRespawnEvent`.
    ///
    /// # Arguments
    /// - `player`: A reference to the player who is respawning.
    /// - `respawn_position`: The position where the player will respawn.
    /// - `yaw`: The yaw angle (horizontal rotation) after respawn.
    /// - `pitch`: The pitch angle (vertical rotation) after respawn.
    ///
    /// # Returns
    /// A new instance of `PlayerRespawnEvent`.
    pub fn new(
        player: Arc<Player>,
        respawn_position: Vector3<f64>,
        yaw: f32,
        pitch: f32, 
    ) -> Self {
        Self {
            player,
            respawn_position,
            yaw,
            pitch,
        }
    }
}

impl PlayerEvent for PlayerRespawnEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}