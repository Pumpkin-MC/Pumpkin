use pumpkin_data::packet::serverbound::play::COMMAND_SUGGESTION;
use pumpkin_macros::java_packet;

use crate::VarInt;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, NetworkReadSliceExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(COMMAND_SUGGESTION)]
pub struct SCommandSuggestion<'a> {
    pub id: VarInt,
    pub command: &'a str,
}

impl<'a> ServerPacket<'a> for SCommandSuggestion<'a> {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            id: bytebuf.get_var_int()?,
            command: bytebuf.get_str_borrowed()?,
        })
    }
}

impl crate::ClientPacket for SCommandSuggestion<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_var_int(&self.id)?;
        write.write_string(self.command)?;
        Ok(())
    }
}
