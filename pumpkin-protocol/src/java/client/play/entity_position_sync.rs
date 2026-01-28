use pumpkin_data::packet::clientbound::PLAY_ENTITY_POSITION_SYNC;
use pumpkin_macros::java_packet;
use pumpkin_util::math::vector3::Vector3;
use serde::Serialize;

use crate::VarInt;

/// Updates the exact position, rotation, and velocity of an entity.
///
/// This packet is used for server-side authority over entity movement.
/// In the latest protocol versions, this replaces several older "Relative Move"
/// packets to provide more precise synchronization and reduce "rubber-banding."
///
/// Note: This packet must NOT be used for the player receiving the packet or
/// any entity the player is currently riding.
#[java_packet(PLAY_ENTITY_POSITION_SYNC)]
#[derive(Serialize)]
pub struct CEntityPositionSync {
    /// The Entity ID of the entity being moved.
    pub entity_id: VarInt,
    /// The absolute position of the entity in the world.
    pub position: Vector3<f64>,
    /// The current velocity (delta) of the entity, used by the client
    /// for smooth interpolation.
    pub delta: Vector3<f64>,
    /// The absolute yaw (horizontal rotation) in degrees.
    pub yaw: f32,
    /// The absolute pitch (vertical rotation) in degrees.
    pub pitch: f32,
    /// Whether the entity is currently touching the ground.
    pub on_ground: bool,
}

impl CEntityPositionSync {
    #[must_use]
    pub fn new(
        entity_id: VarInt,
        position: Vector3<f64>,
        delta: Vector3<f64>,
        yaw: f32,
        pitch: f32,
        on_ground: bool,
    ) -> Self {
        Self {
            entity_id,
            position,
            delta,
            yaw,
            pitch,
            on_ground,
        }
    }
}
