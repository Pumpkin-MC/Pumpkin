use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::PLAY_USE_ITEM;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::VarInt;

#[java_packet(PLAY_USE_ITEM)]
pub struct SUseItem {
    // 0 for main hand, 1 for off hand
    pub hand: VarInt,
    pub sequence: VarInt,
    pub yaw: f32,
    pub pitch: f32,
}

impl<'a> ServerPacket<'a> for SUseItem {
    fn read(
        bytebuf: &mut &'a [u8],
        _protocol_version: &JavaMinecraftVersion,
    ) -> Result<Self, ReadingError> {
        Ok(Self {
            hand: bytebuf.get_var_int()?,
            sequence: bytebuf.get_var_int()?,
            yaw: bytebuf.get_f32_be()?,
            pitch: bytebuf.get_f32_be()?,
        })
    }
}
