use crate::{
    ServerPacket,
    codec::var_int::VarInt,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::PLAY_SPECTATE_ENTITY;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_SPECTATE_ENTITY)]
pub struct SSpectateEntity {
    pub entity_id: Option<VarInt>,
}

impl<'a> ServerPacket<'a> for SSpectateEntity {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            entity_id: bytebuf.get_option(NetworkReadExt::get_var_int)?,
        })
    }
}
