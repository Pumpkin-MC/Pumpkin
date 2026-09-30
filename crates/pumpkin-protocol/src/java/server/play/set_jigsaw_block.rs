use crate::{
    ServerPacket,
    ser::{NetworkReadExt, NetworkReadSliceExt, ReadingError},
};
use pumpkin_data::packet::serverbound::play::SET_JIGSAW_BLOCK;
use pumpkin_macros::java_packet;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::codec::var_int::VarInt;

#[java_packet(SET_JIGSAW_BLOCK)]
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
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let pos = bytebuf.get_block_pos(version)?;
        let name = bytebuf.get_str_bounded_borrowed(32767)?;
        let target = if *version >= JavaMinecraftVersion::V_1_16 {
            bytebuf.get_str_bounded_borrowed(32767)?
        } else {
            ""
        };
        let pool = bytebuf.get_str_bounded_borrowed(32767)?;
        let final_state = bytebuf.get_str_bounded_borrowed(32767)?;
        let joint = if *version >= JavaMinecraftVersion::V_1_16 {
            bytebuf.get_str_bounded_borrowed(32767)?
        } else {
            "aligned"
        };
        let (selection_priority, placement_priority) = if *version >= JavaMinecraftVersion::V_1_20_3
        {
            (bytebuf.get_var_int()?, bytebuf.get_var_int()?)
        } else {
            (VarInt(0), VarInt(0))
        };

        Ok(Self {
            pos,
            name,
            target,
            pool,
            final_state,
            joint,
            selection_priority,
            placement_priority,
        })
    }
}

impl crate::ClientPacket for SSetJigsawBlock<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_block_pos(&self.pos, version)?;
        write.write_string_bounded(self.name, 32767)?;
        if *version >= JavaMinecraftVersion::V_1_16 {
            write.write_string_bounded(self.target, 32767)?;
        }
        write.write_string_bounded(self.pool, 32767)?;
        write.write_string_bounded(self.final_state, 32767)?;
        if *version >= JavaMinecraftVersion::V_1_16 {
            write.write_string_bounded(self.joint, 32767)?;
        }
        if *version >= JavaMinecraftVersion::V_1_20_3 {
            write.write_var_int(&self.selection_priority)?;
            write.write_var_int(&self.placement_priority)?;
        }
        Ok(())
    }
}
