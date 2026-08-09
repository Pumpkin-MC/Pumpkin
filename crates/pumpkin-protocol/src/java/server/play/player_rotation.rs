use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::PLAY_MOVE_PLAYER_ROT;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use super::{FLAG_HORIZONTAL_COLLISION, FLAG_ON_GROUND};

#[java_packet(PLAY_MOVE_PLAYER_ROT)]
pub struct SPlayerRotation {
    pub yaw: f32,
    pub pitch: f32,
    pub ground: bool,
    pub horizontal_collision: bool,
}

impl<'a> ServerPacket<'a> for SPlayerRotation {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let yaw = bytebuf.get_f32_be()?;
        let pitch = bytebuf.get_f32_be()?;
        let collision = bytebuf.get_u8()?;
        Ok(Self {
            yaw,
            pitch,
            ground: collision & FLAG_ON_GROUND != 0,
            horizontal_collision: collision & FLAG_HORIZONTAL_COLLISION != 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_horizontal_collision_separately_from_on_ground() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&45.0f32.to_be_bytes());
        bytes.extend_from_slice(&(-20.0f32).to_be_bytes());
        bytes.push(FLAG_HORIZONTAL_COLLISION);
        let mut input = bytes.as_slice();

        let packet = SPlayerRotation::read(&mut input, &JavaMinecraftVersion::V_26_2).unwrap();

        assert_eq!(packet.yaw, 45.0);
        assert_eq!(packet.pitch, -20.0);
        assert!(!packet.ground);
        assert!(packet.horizontal_collision);
    }
}
