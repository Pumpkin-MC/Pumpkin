use pumpkin_data::packet::serverbound::LOGIN_HELLO;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};

#[java_packet(LOGIN_HELLO)]
pub struct SLoginStart {
    pub name: Box<str>, // 16
    pub uuid: uuid::Uuid,
}

impl<'a> ServerPacket<'a> for SLoginStart {
    fn read(read: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            name: read.get_str_bounded(16)?,
            uuid: read.get_uuid()?,
        })
    }
}
