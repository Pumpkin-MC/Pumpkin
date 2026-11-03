use pumpkin_data::packet::serverbound::login::HELLO;
use pumpkin_macros::java_packet;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};

#[java_packet(HELLO)]
pub struct SLoginStart {
    pub name: Box<str>, // 16
    pub uuid: uuid::Uuid,
}

impl<'a> ServerPacket<'a> for SLoginStart {
    fn read(read: &mut &'a [u8]) -> Result<Self, ReadingError> {
        let name = read.get_str_bounded(16)?;
        let uuid = read.get_uuid()?;

        Ok(Self { name, uuid })
    }
}

impl crate::ClientPacket for SLoginStart {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_string_bounded(&self.name, 16)?;

        write.write_uuid(&self.uuid)?;
        Ok(())
    }
}
