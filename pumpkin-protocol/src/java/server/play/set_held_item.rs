use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::PLAY_SET_CARRIED_ITEM;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_SET_CARRIED_ITEM)]
pub struct SSetHeldItem {
    pub slot: i16,
}

impl<'a> ServerPacket<'a> for SSetHeldItem {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            slot: bytebuf.get_i16_be()?,
        })
    }
}
