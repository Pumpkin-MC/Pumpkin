// Last verified for v2169

use pumpkin_macros::packet;
use pumpkin_util::math::{vector2::Vector2, vector3::Vector3};

use crate::{codec::var_ulong::VarULong, serial::PacketWrite};

/// Sent by the server to correct the client's predicted movement under server authoritative movement.
#[derive(PacketWrite)]
#[packet(161)]
pub struct CCorrectPlayerMovePrediction {
    /// Prediction mode type (e.g. player movement vs vehicle movement).
    pub prediction_type: u8,
    /// Corrected world position coordinates.
    pub pos: Vector3<f32>,
    /// Positional delta velocity vector.
    pub pos_delta: Vector3<f32>,
    /// Pitch and yaw rotation angles.
    pub rotation: Vector2<f32>,
    /// Angular velocity of the vehicle if riding one.
    pub vehicle_angular_velocity: Option<f32>,
    /// Whether the player was resting on solid ground.
    pub on_ground: bool,
    /// Tick number on which the prediction error occurred.
    pub tick: VarULong,
}
