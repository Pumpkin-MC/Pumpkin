use pumpkin_macros::Event;
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player's spawn location is determined.
#[derive(Event, Clone)]
pub struct PlayerSpawnLocationEvent {
    /// The player spawning.
    pub player: Arc<Player>,

    /// The spawn position.
    pub spawn_position: Vector3<f64>,

    /// The spawn world UUID.
    pub world_uuid: uuid::Uuid,
}

impl PlayerSpawnLocationEvent {
    /// Creates a new instance of `PlayerSpawnLocationEvent`.
    pub const fn new(
        player: Arc<Player>,
        spawn_position: Vector3<f64>,
        world_uuid: uuid::Uuid,
    ) -> Self {
        Self {
            player,
            spawn_position,
            world_uuid,
        }
    }
}

impl PlayerEvent for PlayerSpawnLocationEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
