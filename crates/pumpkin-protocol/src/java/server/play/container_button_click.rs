use crate::VarInt;
use pumpkin_data::packet::serverbound::play::CONTAINER_BUTTON_CLICK;
use pumpkin_macros::java_packet;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

#[derive(Debug)]
#[java_packet(CONTAINER_BUTTON_CLICK)]
pub struct SContainerButtonClick {
    pub window_id: VarInt,
    pub button_id: VarInt,
}

impl<'a> ServerPacket<'a> for SContainerButtonClick {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let window_id = bytebuf.get_container_id(version)?;
        let button_id = if *version >= JavaMinecraftVersion::V_1_21_2 {
            bytebuf.get_var_int()?
        } else {
            VarInt(i32::from(bytebuf.get_i8()?))
        };
        Ok(Self {
            window_id,
            button_id,
        })
    }
}

impl crate::ClientPacket for SContainerButtonClick {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_container_id(&self.window_id, version)?;
        if *version >= JavaMinecraftVersion::V_1_21_2 {
            write.write_var_int(&self.button_id)?;
        } else {
            write.write_i8(self.button_id.0 as i8)?;
        }
        Ok(())
    }
}
