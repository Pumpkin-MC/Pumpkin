use pumpkin_data::packet::serverbound::PLAY_ENTITY_TAG_QUERY;
use pumpkin_macros::java_packet;

use crate::{
    ServerPacket,
    codec::var_int::VarInt,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_ENTITY_TAG_QUERY)]
pub struct SEntityTagQuery {
    pub transaction_id: VarInt,
    pub entity_id: VarInt,
}

impl<'a> ServerPacket<'a> for SEntityTagQuery {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            transaction_id: bytebuf.get_var_int()?,
            entity_id: bytebuf.get_var_int()?,
        })
    }
}
