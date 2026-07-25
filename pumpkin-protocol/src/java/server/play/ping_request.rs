use pumpkin_data::packet::serverbound::PLAY_PING_REQUEST;
use pumpkin_macros::java_packet;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_PING_REQUEST)]
pub struct SPlayPingRequest {
    pub payload: i64,
}

impl<'a> ServerPacket<'a> for SPlayPingRequest {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            payload: bytebuf.get_i64_be()?,
        })
    }
}
