use pumpkin_macros::Event;
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player respawns after dying.
///
/// Plugins can modify the respawn position and rotation via blocking handlers.
#[derive(Event, Clone)]
pub struct PlayerRespawnEvent {
    pub player: Arc<Player>,
    pub respawn_position: Vector3<f64>,
    pub respawn_yaw: f32,
    pub respawn_pitch: f32,
    pub is_bed_spawn: bool,
}

impl PlayerRespawnEvent {
    pub fn new(
        player: Arc<Player>,
        respawn_position: Vector3<f64>,
        respawn_yaw: f32,
        respawn_pitch: f32,
        is_bed_spawn: bool,
    ) -> Self {
        Self {
            player,
            respawn_position,
            respawn_yaw,
            respawn_pitch,
            is_bed_spawn,
        }
    }
}

impl PlayerEvent for PlayerRespawnEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
