use pumpkin_data::packet::serverbound::play::SET_COMMAND_MINECART;
use pumpkin_macros::java_packet;

use crate::{
    ServerPacket,
    codec::var_int::VarInt,
    ser::{NetworkReadExt, NetworkReadSliceExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(SET_COMMAND_MINECART)]
pub struct SSetCommandMinecart<'a> {
    pub entity_id: VarInt,
    pub command: &'a str,
    pub track_output: bool,
}

impl<'a> ServerPacket<'a> for SSetCommandMinecart<'a> {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            entity_id: bytebuf.get_var_int()?,
            command: bytebuf.get_str_borrowed()?,
            track_output: bytebuf.get_bool()?,
        })
    }
}

impl crate::ClientPacket for SSetCommandMinecart<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_var_int(&self.entity_id)?;
        write.write_string(self.command)?;
        write.write_bool(self.track_output)?;
        Ok(())
    }
}
