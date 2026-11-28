// Last verified for v2225.

use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

use crate::{codec::var_int::VarInt, serial::PacketWrite};

/// Plays a resource pack sound at the specified position.
#[derive(PacketWrite)]
#[packet(86)]
pub struct CPlaySound<'a> {
    name: &'a str,
    x: VarInt,
    y: VarInt,
    z: VarInt,
    volume: f32,
    pitch: f32,
    loop_count: VarInt,
    bypass_listener_range_check: bool,
    server_sound_handle: Option<u64>,
    playback_position_seconds: Option<f32>,
}

impl<'a> CPlaySound<'a> {
    #[must_use]
    pub fn once(name: &'a str, position: &Vector3<f64>, volume: f32, pitch: f32) -> Self {
        Self {
            name,
            x: VarInt((position.x * 8.0) as i32),
            y: VarInt((position.y * 8.0) as i32),
            z: VarInt((position.z * 8.0) as i32),
            volume,
            pitch,
            loop_count: VarInt(0),
            bypass_listener_range_check: false,
            server_sound_handle: None,
            playback_position_seconds: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_shot_sound_matches_protocol_2225() -> Result<(), Box<dyn std::error::Error>> {
        use crate::serial::PacketRead;

        let packet = CPlaySound::once("note", &Vector3::new(1.25, -0.5, 0.125), 1.0, 0.5);
        let mut bytes = Vec::new();
        packet.write(&mut bytes)?;
        assert_eq!(
            bytes,
            [
                4, b'n', b'o', b't', b'e', 20, 7, 2, // Name and zigzag coordinates.
                0, 0, 128, 63, 0, 0, 0, 63, // Volume and pitch.
                0, 0, 0, 0, // Loop count, bypass, and both option markers.
            ],
        );
        let mut data = &bytes[5..];
        assert_eq!(VarInt::read(&mut data)?.0, 10);
        assert_eq!(VarInt::read(&mut data)?.0, -4);
        assert_eq!(VarInt::read(&mut data)?.0, 1);
        assert_eq!(f32::read(&mut data)?.to_bits(), 1.0f32.to_bits());
        assert_eq!(f32::read(&mut data)?.to_bits(), 0.5f32.to_bits());
        assert_eq!(VarInt::read(&mut data)?.0, 0);
        assert!(!bool::read(&mut data)?);
        assert_eq!(Option::<u64>::read(&mut data)?, None);
        assert_eq!(Option::<f32>::read(&mut data)?, None);
        assert!(data.is_empty());
        Ok(())
    }
}
