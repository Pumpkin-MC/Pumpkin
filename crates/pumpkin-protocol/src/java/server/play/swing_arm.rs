use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::PLAY_SWING;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::VarInt;

#[java_packet(PLAY_SWING)]
pub struct SSwingArm {
    pub hand: VarInt,
}

impl<'a> ServerPacket<'a> for SSwingArm {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            hand: bytebuf.get_var_int()?,
        })
    }
}
