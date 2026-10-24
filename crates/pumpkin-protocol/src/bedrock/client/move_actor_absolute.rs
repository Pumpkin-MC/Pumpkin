// Last verified for v2169

use crate::{codec::var_ulong::VarULong, serial::PacketWrite};
use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

/// Sent by the server to update an entity's absolute position and rotation angles.
#[derive(PacketWrite)]
#[packet(18)]
pub struct CMoveActorAbsolute {
    /// Runtime entity ID of the moving actor.
    pub actor_runtime_id: VarULong,
    /// Bitflags indicating on-ground status, teleportation, or forced movement.
    pub header: u8,
    /// Absolute world position coordinates.
    pub position: Vector3<f32>,
    /// Pitch angle encoded as an 8-bit rotation value (`angle * 256 / 360`).
    pub rotation_x: u8,
    /// Yaw angle encoded as an 8-bit rotation value.
    pub rotation_y: u8,
    /// Head yaw angle encoded as an 8-bit rotation value.
    pub rotation_y_head: u8,
}

impl CMoveActorAbsolute {
    /// Flag set when the entity is resting on solid ground.
    pub const FLAG_ON_GROUND: u8 = 0x01;
    /// Flag indicating the movement is an instantaneous teleport rather than interpolation.
    pub const FLAG_TELEPORT: u8 = 0x02;
    /// Flag indicating movement is forced by the server.
    pub const FLAG_FORCE_MOVE: u8 = 0x04;
}
