use pumpkin_data::packet::serverbound::PLAY_MOVE_PLAYER_POS_ROT;
use pumpkin_macros::java_packet;
use pumpkin_util::math::vector3::Vector3;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

pub const FLAG_ON_GROUND: u8 = 0x01;
pub const FLAG_IN_WALL: u8 = 0x02;

#[java_packet(PLAY_MOVE_PLAYER_POS_ROT)]
pub struct SPlayerPositionRotation {
    pub position: Vector3<f64>,
    pub yaw: f32,
    pub pitch: f32,
    /// bit 0: [`FLAG_ON_GROUND`], bit 1: [`FLAG_IN_WALL`]
    pub collision: u8,
}

impl<'a> ServerPacket<'a> for SPlayerPositionRotation {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            position: Vector3::new(
                bytebuf.get_f64_be()?,
                bytebuf.get_f64_be()?,
                bytebuf.get_f64_be()?,
            ),
            yaw: bytebuf.get_f32_be()?,
            pitch: bytebuf.get_f32_be()?,
            collision: bytebuf.get_u8()?,
        })
    }
}
