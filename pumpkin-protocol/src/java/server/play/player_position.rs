use pumpkin_data::packet::serverbound::PLAY_MOVE_PLAYER_POS;
use pumpkin_macros::java_packet;
use pumpkin_util::math::vector3::Vector3;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_MOVE_PLAYER_POS)]
pub struct SPlayerPosition {
    pub position: Vector3<f64>,
    /// bit 0: [`FLAG_ON_GROUND`], bit 1: [`FLAG_IN_WALL`]
    pub collision: u8,
}

impl<'a> ServerPacket<'a> for SPlayerPosition {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            position: Vector3::new(
                bytebuf.get_f64_be()?,
                bytebuf.get_f64_be()?,
                bytebuf.get_f64_be()?,
            ),
            collision: bytebuf.get_u8()?,
        })
    }
}
