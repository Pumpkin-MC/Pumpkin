use pumpkin_data::packet::serverbound::PLAY_CHAT;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::{
    ClientPacket, ServerPacket,
    codec::var_int::VarInt,
    ser::NetworkWriteExt,
    ser::{NetworkReadExt, NetworkReadSliceExt, ReadingError},
};

#[java_packet(PLAY_CHAT)]
pub struct SChatMessage<'a> {
    pub message: &'a str,
    pub timestamp: i64,
    pub salt: i64,
    pub signature: Option<&'a [u8]>,
    pub message_count: VarInt,
    pub acknowledged: &'a [u8], // Bitset fixed 20 bits
    pub checksum: u8,           // 1.21.5 "fingerprint" checksum
}

impl<'a> ServerPacket<'a> for SChatMessage<'a> {
    fn read(read: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            message: read.get_str_bounded_borrowed(256)?,
            timestamp: read.get_i64_be()?,
            salt: read.get_i64_be()?,
            signature: read.get_option(|v| v.read_slice_borrowed(256))?,
            message_count: read.get_var_int()?,
            acknowledged: read.read_slice_borrowed(3)?,
            checksum: read.get_u8()?,
        })
    }
}

impl ClientPacket for SChatMessage<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_string(self.message)?;
        write.write_i64_be(self.timestamp)?;
        write.write_i64_be(self.salt)?;
        write.write_option(&self.signature, |p, v| p.write_slice(v))?;
        write.write_var_int(&self.message_count)?;
        write.write_slice(self.acknowledged)?;
        write.write_u8(self.checksum)?;

        Ok(())
    }
}
