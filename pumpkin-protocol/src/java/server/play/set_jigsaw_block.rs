use crate::{
    ServerPacket,
    ser::{NetworkReadExt, NetworkReadSliceExt, ReadingError},
};
use pumpkin_data::packet::serverbound::PLAY_SET_JIGSAW_BLOCK;
use pumpkin_macros::java_packet;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::codec::var_int::VarInt;

#[java_packet(PLAY_SET_JIGSAW_BLOCK)]
pub struct SSetJigsawBlock<'a> {
    pub pos: BlockPos,
    pub name: &'a str,
    pub target: &'a str,
    pub pool: &'a str,
    pub final_state: &'a str,
    pub joint: &'a str,
    pub selection_priority: VarInt,
    pub placement_priority: VarInt,
}

impl<'a> ServerPacket<'a> for SSetJigsawBlock<'a> {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            pos: BlockPos::from_i64(bytebuf.get_i64_be()?),
            name: bytebuf.get_str_bounded_borrowed(32767)?,
            target: bytebuf.get_str_bounded_borrowed(32767)?,
            pool: bytebuf.get_str_bounded_borrowed(32767)?,
            final_state: bytebuf.get_str_bounded_borrowed(32767)?,
            joint: bytebuf.get_str_bounded_borrowed(32767)?,
            selection_priority: bytebuf.get_var_int()?,
            placement_priority: bytebuf.get_var_int()?,
        })
    }
}
