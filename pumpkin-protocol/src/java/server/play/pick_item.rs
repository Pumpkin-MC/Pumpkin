use pumpkin_data::packet::serverbound::{PLAY_PICK_ITEM_FROM_BLOCK, PLAY_PICK_ITEM_FROM_ENTITY};
use pumpkin_macros::java_packet;
use pumpkin_util::math::position::BlockPos;

use crate::codec::var_int::VarInt;
use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_PICK_ITEM_FROM_BLOCK)]
pub struct SPickItemFromBlock {
    pub pos: BlockPos,
    pub include_data: bool,
}

impl<'a> ServerPacket<'a> for SPickItemFromBlock {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            pos: BlockPos::from_i64(bytebuf.get_i64_be()?),
            include_data: bytebuf.get_bool()?,
        })
    }
}

#[java_packet(PLAY_PICK_ITEM_FROM_ENTITY)]
pub struct SPickItemFromEntity {
    pub id: VarInt,
    pub include_data: bool,
}

impl<'a> ServerPacket<'a> for SPickItemFromEntity {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            id: bytebuf.get_var_int()?,
            include_data: bytebuf.get_bool()?,
        })
    }
}
