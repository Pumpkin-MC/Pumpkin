use pumpkin_data::packet::serverbound::PLAY_SPECTATE_ENTITY;
use pumpkin_macros::java_packet;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_SPECTATE_ENTITY)]
pub struct SSpectateEntity {
    pub target: uuid::Uuid,
}

impl<'a> ServerPacket<'a> for SSpectateEntity {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            target: bytebuf.get_uuid()?,
        })
    }
}
