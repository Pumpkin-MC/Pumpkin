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
    fn read(read: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let message = read.get_str_bounded_borrowed(256)?;
        let timestamp = read.get_i64_be()?;
        let salt = read.get_i64_be()?;
        let signature = read.get_option(|v| v.read_slice_borrowed(256))?;
        let message_count = read.get_var_int()?;
        let acknowledged = read.read_slice_borrowed(3)?;
        let checksum = if version >= &JavaMinecraftVersion::V_1_21_5 {
            read.get_u8()?
        } else {
            0
        };

        Ok(Self {
            message,
            timestamp,
            salt,
            signature,
            message_count,
            acknowledged,
            checksum,
        })
    }
}

impl ClientPacket for SChatMessage<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_string(self.message)?;
        write.write_i64_be(self.timestamp)?;
        write.write_i64_be(self.salt)?;
        write.write_option(&self.signature, |p, v| p.write_slice(v))?;
        write.write_var_int(&self.message_count)?;
        write.write_slice(self.acknowledged)?;
        if version >= &JavaMinecraftVersion::V_1_21_5 {
            write.write_u8(self.checksum)?;
        }

        Ok(())
    }
}
