use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

use crate::{entity::player::Player, world::World};

use super::PlayerEvent;

/// An event that occurs when a player gets teleported to another world.
///
/// This event contains information about the player changing worlds.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerChangeWorldEvent {
    /// The player who is joining the game.
    pub player: Arc<Player>,

    /// The previous world the player was in.
    pub previous_world: Arc<World>,

    /// The new world the player is in.
    pub new_world: Arc<World>,

    /// Position the player is teleported to.
    pub position: Vector3<f64>,

    /// Yaw of the player after teleportation.
    pub yaw: f32,

    /// Pitch of the player after teleportation.
    pub pitch: f32,
}

impl PlayerChangeWorldEvent {
    /// Creates a new instance of `PlayerChangeWorldEvent`.
    ///
    /// # Arguments
    /// - `player`: A reference to the player changing worlds.
    /// - `previous_world`: The previous world the player was in.
    /// - `new_world`: The new world the player is in.
    /// - `position`: Position the player is teleported to.
    /// - `yaw`: Yaw of the player after teleportation.
    /// - `pitch`: Pitch of the player after teleportation.
    ///
    /// # Returns
    /// A new instance of `PlayerChangeWorldEvent`.
    pub fn new(
        player: Arc<Player>,
        previous_world: Arc<World>,
        new_world: Arc<World>,
        position: Vector3<f64>,
        yaw: f32,
        pitch: f32,
    ) -> Self {
        Self {
            player,
            previous_world,
            new_world,
            position,
            yaw,
            pitch,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerChangeWorldEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
