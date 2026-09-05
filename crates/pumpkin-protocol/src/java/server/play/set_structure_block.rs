use pumpkin_data::packet::serverbound::play::SET_STRUCTURE_BLOCK;
use pumpkin_macros::java_packet;

use crate::{
    ServerPacket,
    codec::{var_int::VarInt, var_long::VarLong},
    ser::{NetworkReadExt, NetworkReadSliceExt, ReadingError},
};
use pumpkin_util::{math::position::BlockPos, version::JavaMinecraftVersion};

#[java_packet(SET_STRUCTURE_BLOCK)]
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

impl SSetStructureBlock<'_> {
    pub const ACTION_UPDATE_DATA: i32 = 0;
    pub const ACTION_SAVE_AREA: i32 = 1;
    pub const ACTION_LOAD_AREA: i32 = 2;
    pub const ACTION_SCAN_AREA: i32 = 3;

    pub const MODE_SAVE: i32 = 0;
    pub const MODE_LOAD: i32 = 1;
    pub const MODE_CORNER: i32 = 2;
    pub const MODE_DATA: i32 = 3;

    pub const FLAG_IGNORE_ENTITIES: u8 = 0x01;
    pub const FLAG_SHOW_AIR: u8 = 0x02;
    pub const FLAG_SHOW_BOUNDING_BOX: u8 = 0x04;

    #[must_use]
    pub const fn ignore_entities(&self) -> bool {
        (self.flags & Self::FLAG_IGNORE_ENTITIES) != 0
    }

    #[must_use]
    pub const fn show_air(&self) -> bool {
        (self.flags & Self::FLAG_SHOW_AIR) != 0
    }

    #[must_use]
    pub const fn show_bounding_box(&self) -> bool {
        (self.flags & Self::FLAG_SHOW_BOUNDING_BOX) != 0
    }
}

impl<'a> ServerPacket<'a> for SSetStructureBlock<'a> {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            location: bytebuf.get_block_pos(version)?,
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
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_block_pos(&self.location, version)?;
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
