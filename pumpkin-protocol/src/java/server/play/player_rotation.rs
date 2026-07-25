use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::PLAY_MOVE_PLAYER_ROT;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_MOVE_PLAYER_ROT)]
pub struct SPlayerRotation {
    pub yaw: f32,
    pub pitch: f32,
    pub ground: bool,
}

impl<'a> ServerPacket<'a> for SPlayerRotation {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            yaw: bytebuf.get_f32_be()?,
            pitch: bytebuf.get_f32_be()?,
            ground: bytebuf.get_bool()?,
        })
    }
}
