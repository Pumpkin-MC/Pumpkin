use pumpkin_data::packet::serverbound::PLAY_CHAT_COMMAND;
use pumpkin_macros::java_packet;

use crate::{
    ServerPacket,
    ser::{NetworkReadSliceExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_CHAT_COMMAND)]
pub struct SChatCommand<'a> {
    pub command: &'a str,
}

impl<'a> ServerPacket<'a> for SChatCommand<'a> {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            command: bytebuf.get_str_borrowed()?,
        })
    }
}
