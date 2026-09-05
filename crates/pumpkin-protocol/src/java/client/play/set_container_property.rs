use pumpkin_data::packet::clientbound::play::CONTAINER_SET_DATA;
use pumpkin_macros::java_packet;

use crate::VarInt;
use crate::ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError};
use crate::{ClientPacket, ServerPacket};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(CONTAINER_SET_DATA)]
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
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_container_id(&self.window_id, version)?;
        write.write_i16_be(self.property)?;
        write.write_i16_be(self.value)?;
        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CSetContainerProperty {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let window_id = bytebuf.get_container_id(version)?;
        let property = bytebuf.get_i16_be()?;
        let value = bytebuf.get_i16_be()?;
        Ok(Self {
            window_id,
            property,
            value,
        })
    }
}
