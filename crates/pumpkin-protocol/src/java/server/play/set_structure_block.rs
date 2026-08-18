use pumpkin_data::packet::serverbound::PLAY_SET_STRUCTURE_BLOCK;
use pumpkin_macros::java_packet;

use crate::{
    ServerPacket,
    codec::{var_int::VarInt, var_long::VarLong},
    ser::{NetworkReadExt, NetworkReadSliceExt, ReadingError},
};
use pumpkin_util::{math::position::BlockPos, version::JavaMinecraftVersion};

#[java_packet(PLAY_SET_STRUCTURE_BLOCK)]
pub struct SSetStructureBlock<'a> {
    pub location: BlockPos,
    pub action: VarInt,
    pub mode: VarInt,
    pub name: &'a str,
    pub offset_x: i8,
    pub offset_y: i8,
    pub offset_z: i8,
    pub size_x: u8,
    pub size_y: u8,
    pub size_z: u8,
    pub mirror: VarInt,
    pub rotation: VarInt,
    pub metadata: &'a str,
    pub integrity: f32,
    pub seed: VarLong,
    pub flags: u8,
}

impl<'a> ServerPacket<'a> for SSetStructureBlock<'a> {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            location: BlockPos::from_i64(bytebuf.get_i64_be()?),
            action: bytebuf.get_var_int()?,
            mode: bytebuf.get_var_int()?,
            name: bytebuf.get_str_borrowed()?,
            offset_x: bytebuf.get_i8()?,
            offset_y: bytebuf.get_i8()?,
            offset_z: bytebuf.get_i8()?,
            size_x: bytebuf.get_u8()?,
            size_y: bytebuf.get_u8()?,
            size_z: bytebuf.get_u8()?,
            mirror: bytebuf.get_var_int()?,
            rotation: bytebuf.get_var_int()?,
            metadata: bytebuf.get_str_borrowed()?,
            integrity: bytebuf.get_f32()?,
            seed: bytebuf.get_var_long()?,
            flags: bytebuf.get_u8()?,
        })
    }
}

impl crate::ClientPacket for SSetStructureBlock<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_block_pos(&self.location)?;
        write.write_var_int(&self.action)?;
        write.write_var_int(&self.mode)?;
        write.write_string(self.name)?;
        write.write_i8(self.offset_x)?;
        write.write_i8(self.offset_y)?;
        write.write_i8(self.offset_z)?;
        write.write_u8(self.size_x)?;
        write.write_u8(self.size_y)?;
        write.write_u8(self.size_z)?;
        write.write_var_int(&self.mirror)?;
        write.write_var_int(&self.rotation)?;
        write.write_string(self.metadata)?;
        write.write_f32_be(self.integrity)?;
        write.write_var_long(&self.seed)?;
        write.write_u8(self.flags)?;
        Ok(())
    }
}
