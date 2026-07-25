use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::PLAY_JIGSAW_GENERATE;
use pumpkin_macros::java_packet;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::codec::var_int::VarInt;

#[java_packet(PLAY_JIGSAW_GENERATE)]
pub struct SJigsawGenerate {
    pub pos: BlockPos,
    pub levels: VarInt,
    pub keep_jigsaws: bool,
}

impl<'a> ServerPacket<'a> for SJigsawGenerate {
    fn read(
        bytebuf: &mut &'a [u8],
        _protocol_version: &JavaMinecraftVersion,
    ) -> Result<Self, ReadingError> {
        Ok(Self {
            pos: BlockPos::from_i64(bytebuf.get_i64_be()?),
            levels: bytebuf.get_var_int()?,
            keep_jigsaws: bytebuf.get_bool()?,
        })
    }
}
