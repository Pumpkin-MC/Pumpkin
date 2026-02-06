use pumpkin_macros::Event;
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player respawns.
#[derive(Event, Clone)]
pub struct PlayerRespawnEvent {
    /// The player who is respawning.
    pub player: Arc<Player>,

    /// The respawn position.
    pub respawn_position: Vector3<f64>,

    /// The world UUID to respawn in.
    pub world_uuid: uuid::Uuid,

    /// Whether this was a bed spawn.
    pub is_bed_spawn: bool,

    /// Whether this was an anchor spawn.
    pub is_anchor_spawn: bool,

    /// Whether the respawn block was missing.
    pub is_missing_respawn_block: bool,

    /// Respawn reason.
    pub reason: String,
}

impl PlayerRespawnEvent {
    /// Creates a new instance of `PlayerRespawnEvent`.
    pub const fn new(
        player: Arc<Player>,
        respawn_position: Vector3<f64>,
        world_uuid: uuid::Uuid,
        is_bed_spawn: bool,
        is_anchor_spawn: bool,
        is_missing_respawn_block: bool,
        reason: String,
    ) -> Self {
        Self {
            player,
            respawn_position,
            world_uuid,
            is_bed_spawn,
            is_anchor_spawn,
            is_missing_respawn_block,
            reason,
        }
    }
}

impl PlayerEvent for PlayerRespawnEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
