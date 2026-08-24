use pumpkin_data::packet::serverbound::play::{PICK_ITEM_FROM_BLOCK, PICK_ITEM_FROM_ENTITY};
use pumpkin_macros::java_packet;
use pumpkin_util::math::position::BlockPos;

use crate::codec::var_int::VarInt;
use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PICK_ITEM_FROM_BLOCK)]
pub struct SPickItemFromBlock {
    pub pos: BlockPos,
    pub include_data: bool,
}

impl<'a> ServerPacket<'a> for SPickItemFromBlock {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            pos: bytebuf.get_block_pos(version)?,
            include_data: bytebuf.get_bool()?,
        })
    }
}

impl crate::ClientPacket for SPickItemFromBlock {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_block_pos(&self.pos, version)?;
        write.write_bool(self.include_data)?;
        Ok(())
    }
}

#[java_packet(PICK_ITEM_FROM_ENTITY)]
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

impl crate::ClientPacket for SPickItemFromEntity {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_var_int(&self.id)?;
        write.write_bool(self.include_data)?;
        Ok(())
    }
}
