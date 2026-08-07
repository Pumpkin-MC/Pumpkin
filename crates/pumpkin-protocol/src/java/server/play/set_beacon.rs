use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::PLAY_SET_BEACON;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::codec::var_int::VarInt;

#[java_packet(PLAY_SET_BEACON)]
pub struct SSetBeacon {
    pub primary_effect: Option<VarInt>,
    pub secondary_effect: Option<VarInt>,
}

impl<'a> ServerPacket<'a> for SSetBeacon {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            primary_effect: bytebuf.get_option(NetworkReadExt::get_var_int)?,
            secondary_effect: bytebuf.get_option(NetworkReadExt::get_var_int)?,
        })
    }
}
