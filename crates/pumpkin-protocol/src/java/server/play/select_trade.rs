use pumpkin_data::packet::serverbound::play::SELECT_TRADE;
use pumpkin_macros::java_packet;

use crate::VarInt;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};

#[java_packet(SELECT_TRADE)]
pub struct SSelectTrade {
    pub selected_slot: VarInt,
}

impl<'a> ServerPacket<'a> for SSelectTrade {
    fn read(bytebuf: &mut &'a [u8]) -> Result<Self, ReadingError> {
        Ok(Self {
            selected_slot: bytebuf.get_var_int()?,
        })
    }
}

impl crate::ClientPacket for SSelectTrade {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_var_int(&self.selected_slot)?;
        Ok(())
    }
}
