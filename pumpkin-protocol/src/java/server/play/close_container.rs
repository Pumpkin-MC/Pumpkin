use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::PLAY_CONTAINER_CLOSE;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::VarInt;

#[java_packet(PLAY_CONTAINER_CLOSE)]
pub struct SCloseContainer {
    pub window_id: VarInt,
}

impl<'a> ServerPacket<'a> for SCloseContainer {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            window_id: bytebuf.get_var_int()?,
        })
    }
}
