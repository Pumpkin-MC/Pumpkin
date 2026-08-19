use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::PLAY_CLIENT_COMMAND;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::VarInt;

#[java_packet(PLAY_CLIENT_COMMAND)]
pub struct SClientCommand {
    pub action_id: VarInt,
}

impl<'a> ServerPacket<'a> for SClientCommand {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            action_id: bytebuf.get_var_int()?,
        })
    }
}

impl crate::ClientPacket for SClientCommand {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_var_int(&self.action_id)?;
        Ok(())
    }
}
