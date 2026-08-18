use pumpkin_data::packet::serverbound::PLAY_SELECT_TRADE;
use pumpkin_macros::java_packet;

use crate::VarInt;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_SELECT_TRADE)]
pub struct SSelectTrade {
    pub selected_slot: VarInt,
}

impl<'a> ServerPacket<'a> for SSelectTrade {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            selected_slot: bytebuf.get_var_int()?,
        })
    }
}

impl crate::ClientPacket for SSelectTrade {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_var_int(&self.selected_slot)?;
        Ok(())
    }
}
