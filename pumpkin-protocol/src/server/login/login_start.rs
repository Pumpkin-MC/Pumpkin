use crate::ser::{NetworkRead, NetworkWrite, WritingError};
use pumpkin_data::packet::serverbound::LOGIN_HELLO;
use pumpkin_macros::packet;

use crate::{ClientPacket, ServerPacket, ser::ReadingError};

#[packet(LOGIN_HELLO)]
pub struct SLoginStart {
    pub name: String, // 16
    pub uuid: uuid::Uuid,
}

impl ClientPacket for SLoginStart {
    fn write(&self, write: impl NetworkWrite) -> Result<(), WritingError> {
        let mut write = write;

        write.write_string_bounded(&self.name, 16)?;
        write.write_uuid(&self.uuid)
    }
}

impl ServerPacket for SLoginStart {
    fn read(read: impl NetworkRead) -> Result<Self, ReadingError> {
        let mut read = read;

        Ok(Self {
            name: read.get_string_bounded(16)?,
            uuid: read.get_uuid()?,
        })
    }
}
