use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::PLAY_TELEPORT_TO_ENTITY;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_TELEPORT_TO_ENTITY)]
pub struct STeleportToEntity {
    pub target: uuid::Uuid,
}

impl<'a> ServerPacket<'a> for STeleportToEntity {
    fn read(
        bytebuf: &mut &'a [u8],
        _protocol_version: &JavaMinecraftVersion,
    ) -> Result<Self, ReadingError> {
        Ok(Self {
            target: bytebuf.get_uuid()?,
        })
    }
}
