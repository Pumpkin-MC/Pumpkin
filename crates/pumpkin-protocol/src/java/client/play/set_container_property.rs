use pumpkin_data::packet::clientbound::PLAY_CONTAINER_SET_DATA;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::VarInt;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_CONTAINER_SET_DATA)]
pub struct CSetContainerProperty {
    pub window_id: VarInt,
    pub property: i16,
    pub value: i16,
}

impl CSetContainerProperty {
    #[must_use]
    pub const fn new(window_id: VarInt, property: i16, value: i16) -> Self {
        Self {
            window_id,
            property,
            value,
        }
    }
}

impl ClientPacket for CSetContainerProperty {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.window_id)?;
        write.write_i16(self.property)?;
        write.write_i16(self.value)?;
        Ok(())
    }
}
