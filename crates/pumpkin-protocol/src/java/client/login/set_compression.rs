use pumpkin_data::packet::clientbound::login::LOGIN_COMPRESSION;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::VarInt;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

/// Sent by the server to enable network compression for all subsequent packets.
///
/// Once this packet is sent, both the server and the client must compress any
/// packet with a size equal to or greater than the specified threshold.
#[java_packet(LOGIN_COMPRESSION)]
pub struct CSetCompression {
    /// The packet size threshold (in bytes) at which compression is applied.
    ///
    /// Packets smaller than this are sent uncompressed. A negative threshold
    /// typically disables compression.
    pub threshold: VarInt,
}

impl CSetCompression {
    #[must_use]
    pub const fn new(threshold: VarInt) -> Self {
        Self { threshold }
    }
}

impl ClientPacket for CSetCompression {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.threshold)?;
        Ok(())
    }
}

impl<'a> crate::ServerPacket<'a> for CSetCompression {
    fn read(
        read: &mut &'a [u8],
        _version: &JavaMinecraftVersion,
    ) -> Result<Self, crate::ReadingError> {
        use crate::ser::NetworkReadExt;
        Ok(Self {
            threshold: read.get_var_int()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ServerPacket;

    #[test]
    fn set_compression_roundtrip() {
        let packet = CSetCompression::new(crate::VarInt(256));
        let mut buf = Vec::new();
        let version = JavaMinecraftVersion::V_1_21_4;
        packet.write_packet_data(&mut buf, &version).unwrap();

        let mut slice = buf.as_slice();
        let read_packet = CSetCompression::read(&mut slice, &version).unwrap();
        assert_eq!(read_packet.threshold.0, 256);
    }
}
