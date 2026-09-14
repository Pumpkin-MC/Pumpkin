use crate::{
    ClientPacket, ServerPacket, VarInt,
    ser::{NetworkReadExt, NetworkReadSliceExt, NetworkWriteExt, ReadingError, WritingError},
};
use pumpkin_data::packet::clientbound::play::DEBUG_ENTITY_VALUE;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(DEBUG_ENTITY_VALUE)]
pub struct CDebugEntityValue<'a> {
    pub entity_id: VarInt,
    pub name: &'a str,
    pub value: &'a str,
}

impl<'a> CDebugEntityValue<'a> {
    #[must_use]
    pub const fn new(entity_id: VarInt, name: &'a str, value: &'a str) -> Self {
        Self {
            entity_id,
            name,
            value,
        }
    }
}

impl ClientPacket for CDebugEntityValue<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_var_int(&self.entity_id)?;
        write.write_string(self.name)?;
        write.write_string(self.value)?;
        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CDebugEntityValue<'a> {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            entity_id: bytebuf.get_var_int()?,
            name: bytebuf.get_str_borrowed()?,
            value: bytebuf.get_str_borrowed()?,
        })
    }
}
