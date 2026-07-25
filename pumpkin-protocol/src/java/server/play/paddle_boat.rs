use pumpkin_data::packet::serverbound::PLAY_PADDLE_BOAT;
use pumpkin_macros::java_packet;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_PADDLE_BOAT)]
pub struct SPaddleBoat {
    pub left_paddle: bool,
    pub right_paddle: bool,
}

impl<'a> ServerPacket<'a> for SPaddleBoat {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            left_paddle: bytebuf.get_bool()?,
            right_paddle: bytebuf.get_bool()?,
        })
    }
}
