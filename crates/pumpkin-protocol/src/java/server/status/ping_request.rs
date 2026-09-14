use crate::{
    ServerPacket,
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError},
};
use pumpkin_data::packet::serverbound::status::PING_REQUEST;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

/// Sent by the client to measure the round-trip time (latency) to the server.
///
/// This is the second part of the Server List Ping (SLP) process
/// The server should respond with `CPingResponse`.
#[java_packet(PING_REQUEST)]
pub struct SStatusPingRequest {
    pub payload: i64,
}

impl<'a> ServerPacket<'a> for SStatusPingRequest {
    fn read(
        bytebuf: &mut &'a [u8],
        _protocol_version: &JavaMinecraftVersion,
    ) -> Result<Self, ReadingError> {
        Ok(Self {
            payload: bytebuf.get_i64_be()?,
        })
    }
}

impl crate::ClientPacket for SStatusPingRequest {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_i64_be(self.payload)?;
        Ok(())
    }
}
