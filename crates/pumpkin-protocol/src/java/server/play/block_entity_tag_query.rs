use pumpkin_data::packet::serverbound::play::BLOCK_ENTITY_TAG_QUERY;
use pumpkin_macros::java_packet;

use crate::{
    ServerPacket,
    codec::var_int::VarInt,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_util::{math::position::BlockPos, version::JavaMinecraftVersion};

#[java_packet(BLOCK_ENTITY_TAG_QUERY)]
pub struct SBlockEntityTagQuery {
    pub transaction_id: VarInt,
    pub location: BlockPos,
}

impl<'a> ServerPacket<'a> for SBlockEntityTagQuery {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            transaction_id: bytebuf.get_var_int()?,
            location: bytebuf.get_block_pos(version)?,
        })
    }
}

impl crate::ClientPacket for SBlockEntityTagQuery {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_var_int(&self.transaction_id)?;
        write.write_block_pos(&self.location, version)?;
        Ok(())
    }
}
