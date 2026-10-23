use pumpkin_data::packet::serverbound::play::CHAT_COMMAND;
use pumpkin_macros::java_packet;

use crate::{
    ServerPacket,
    ser::{NetworkReadSliceExt, ReadingError},
};

#[java_packet(CHAT_COMMAND)]
pub struct SChatCommand<'a> {
    pub command: &'a str,
}

impl<'a> ServerPacket<'a> for SChatCommand<'a> {
    fn read(bytebuf: &mut &'a [u8]) -> Result<Self, ReadingError> {
        Ok(Self {
            command: bytebuf.get_str_borrowed()?,
        })
    }
}

impl crate::ClientPacket for SChatCommand<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_string(self.command)?;
        Ok(())
    }
}
