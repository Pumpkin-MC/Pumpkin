use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::play::CONTAINER_CLOSE;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::VarInt;

#[java_packet(CONTAINER_CLOSE)]
pub struct SCloseContainer {
    pub window_id: VarInt,
}

impl<'a> ServerPacket<'a> for SCloseContainer {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let window_id = if *version >= JavaMinecraftVersion::V_1_21_2 {
            bytebuf.get_var_int()?
        } else {
            VarInt(bytebuf.get_u8()? as i32)
        };

        Ok(Self { window_id })
    }
}

impl crate::ClientPacket for SCloseContainer {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;

        if *version >= JavaMinecraftVersion::V_1_21_2 {
            write.write_var_int(&self.window_id)?;
        } else {
            write.write_u8(self.window_id.0 as u8)?;
        }

        Ok(())
    }
}
