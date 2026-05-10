use std::io::Read;

use crate::{
    ClientPacket, ConnectionState, ReadingError, ServerPacket, VarInt, ser::NetworkReadExt,
    ser::NetworkWriteExt,
};
use pumpkin_data::packet::serverbound::HANDSHAKE_INTENTION;
use pumpkin_macros::java_packet;
use pumpkin_util::version::MinecraftVersion;

/// The very first packet sent by the client to initiate a connection
///
/// It determines whether the client wants to check the server status (SLP)
/// or actually login to play.
#[java_packet(HANDSHAKE_INTENTION)]
pub struct SHandShake {
    /// The protocol version of the client (e.g., 767 for 1.21).
    pub protocol_version: VarInt,
    /// The hostname or IP used by the client to connect
    pub server_address: String,
    /// The port number used by the client to connect
    pub server_port: u16,
    /// The state the client wants to transition to (1 for Status, 2 for Login)
    pub next_state: ConnectionState,
}

impl ServerPacket for SHandShake {
    fn read(mut read: impl Read, _version: &MinecraftVersion) -> Result<Self, ReadingError> {
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

impl ClientPacket for SHandShake {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &MinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.protocol_version)?;
        write.write_string(&self.server_address)?;
        write.write_u16_be(self.server_port)?;
        write.write_var_int(&VarInt(self.next_state as i32))?;
        Ok(())
    }
}
