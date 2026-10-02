use crate::{ServerPacket, ser::ReadingError};
use pumpkin_data::packet::serverbound::status::STATUS_REQUEST;
use pumpkin_macros::java_packet;

/// Sent by the client to request the server's current status information.
///
/// This is the first packet sent during the "Status" state.
/// The server should respond with `CStatusResponse`.
#[java_packet(STATUS_REQUEST)]
pub struct SStatusRequest;

impl<'a> ServerPacket<'a> for SStatusRequest {
    fn read(_bytebuf: &mut &'a [u8]) -> Result<Self, ReadingError> {
        Ok(Self)
    }
}

impl crate::ClientPacket for SStatusRequest {
    fn write_packet_data(
        &self,
        _write: impl std::io::Write,
    ) -> Result<(), crate::ser::WritingError> {
        Ok(())
    }
}
