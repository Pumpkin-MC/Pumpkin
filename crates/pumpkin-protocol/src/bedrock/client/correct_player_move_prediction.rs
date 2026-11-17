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

#[cfg(test)]
mod tests {
    use super::*;

    /// order: type, pos, delta, vehicle rotation, optional angular velocity,
    /// on ground, unsigned varlong tick.
    #[test]
    fn player_correction_layout() {
        let packet = CCorrectPlayerMovePrediction {
            prediction_type: 0,
            pos: Vector3::new(1.0, 2.0, 3.0),
            pos_delta: Vector3::new(0.5, 0.0, -0.5),
            rotation: Vector2::default(),
            vehicle_angular_velocity: None,
            on_ground: true,
            tick: VarULong(300),
        };
        let mut buf = Vec::new();
        packet.write(&mut buf).unwrap();

        let mut expected = vec![0u8];
        for f in [1.0f32, 2.0, 3.0, 0.5, 0.0, -0.5, 0.0, 0.0] {
            expected.extend_from_slice(&f.to_le_bytes());
        }
        expected.extend_from_slice(&[0, 1, 0xAC, 0x02]);
        assert_eq!(buf, expected);
    }
}
