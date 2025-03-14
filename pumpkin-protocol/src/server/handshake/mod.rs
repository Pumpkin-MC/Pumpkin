use crate::ser::{NetworkRead, NetworkWrite, WritingError};
use crate::{ClientPacket, ConnectionState, ServerPacket, VarInt, ser::ReadingError};
use pumpkin_data::packet::serverbound::HANDSHAKE_INTENTION;
use pumpkin_macros::packet;

#[packet(HANDSHAKE_INTENTION)]
pub struct SHandShake {
    pub protocol_version: VarInt,
    pub server_address: String, // 255
    pub server_port: u16,
    pub next_state: ConnectionState,
}

impl ClientPacket for SHandShake {
    fn write(&self, write: impl NetworkWrite) -> Result<(), WritingError> {
        let mut write = write;

        write.write_var_int(&self.protocol_version)?;
        write.write_string_bounded(&self.server_address, 255)?;
        write.write_u16_be(self.server_port)?;
        write.write_var_int(&VarInt(self.next_state as i32))
    }
}

impl ServerPacket for SHandShake {
    fn read(read: impl NetworkRead) -> Result<Self, ReadingError> {
        let mut read = read;

        Ok(Self {
            protocol_version: read.get_var_int()?,
            server_address: read.get_string_bounded(255)?,
            server_port: read.get_u16_be()?,
            next_state: read
                .get_var_int()?
                .try_into()
                .map_err(|_| ReadingError::Message("Invalid Status".to_string()))?,
        })
    }
}
