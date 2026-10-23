use pumpkin_data::packet::serverbound::play::CHAT_ACK;
use pumpkin_macros::java_packet;

use crate::{
    ServerPacket,
    codec::var_int::VarInt,
    ser::{NetworkReadExt, ReadingError},
};

#[java_packet(CHAT_ACK)]
pub struct SChatAck {
    pub offset: VarInt,
}

impl<'a> ServerPacket<'a> for SChatAck {
    fn read(bytebuf: &mut &'a [u8]) -> Result<Self, ReadingError> {
        let offset = bytebuf.get_var_int()?;

        Ok(Self { offset })
    }
}

impl crate::ClientPacket for SChatAck {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_var_int(&self.offset)?;
        Ok(())
    }
}
