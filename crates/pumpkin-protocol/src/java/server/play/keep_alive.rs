use pumpkin_data::packet::serverbound::PLAY_KEEP_ALIVE;
use pumpkin_macros::java_packet;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_KEEP_ALIVE)]
pub struct SKeepAlive {
    pub keep_alive_id: i64,
}

impl<'a> ServerPacket<'a> for SKeepAlive {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            keep_alive_id: bytebuf.get_i64_be()?,
        })
    }
}
