use std::io::Read;

use crate::{ConnectionState, ReadingError, ServerPacket, VarInt, ser::NetworkReadExt};
use pumpkin_data::packet::serverbound::HANDSHAKE_INTENTION;
use pumpkin_macros::packet;

#[packet(HANDSHAKE_INTENTION)]
pub struct SHandShake {
    pub protocol_version: VarInt,
    pub server_address: String, // 255
    pub server_port: u16,
    pub next_state: ConnectionState,
}

impl ServerPacket for SHandShake {
    fn read(read: impl Read) -> Result<Self, ReadingError> {
        let mut read = read;
        Ok(Self {
            protocol_version: read.get_var_int()?,
            server_address: read.get_string_bounded(255)?,
            server_port: read.get_u16_be()?,
            next_state: read
                .get_var_int()?
                .try_into()
                .map_err(|_| ReadingError::Message("Invalid status".to_string()))?,
        })
    }
}
