use pumpkin_data::packet::serverbound::play::CHAT_COMMAND_SIGNED;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::{
    ClientPacket, ServerPacket,
    codec::var_int::VarInt,
    ser::{NetworkReadExt, NetworkReadSliceExt, NetworkWriteExt, ReadingError, WritingError},
};

pub struct ArgumentSignature<'a> {
    pub name: &'a str,
    pub signature: &'a [u8],
}

#[java_packet(CHAT_COMMAND_SIGNED)]
pub struct SChatCommandSigned<'a> {
    pub command: &'a str,
    pub timestamp: i64,
    pub salt: i64,
    pub argument_signatures: Vec<ArgumentSignature<'a>>,
    pub message_count: VarInt,
    pub acknowledged: &'a [u8],
    pub checksum: u8,
}

impl<'a> ServerPacket<'a> for SChatCommandSigned<'a> {
    fn read(read: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let command = read.get_str_bounded_borrowed(256)?;
        let timestamp = read.get_i64_be()?;
        let salt = read.get_i64_be()?;
        let arg_count = read.get_var_int()?.0 as usize;
        let mut argument_signatures = Vec::with_capacity(arg_count);
        for _ in 0..arg_count {
            let name = read.get_str_bounded_borrowed(16)?;
            let signature = read.read_slice_borrowed(256)?;
            argument_signatures.push(ArgumentSignature { name, signature });
        }
        let message_count = read.get_var_int()?;
        let acknowledged = read.read_slice_borrowed(3)?;
        let checksum = if *version >= JavaMinecraftVersion::V_1_21_5 {
            read.get_u8()?
        } else {
            0
        };

        Ok(Self {
            command,
            timestamp,
            salt,
            argument_signatures,
            message_count,
            acknowledged,
            checksum,
        })
    }
}

impl ClientPacket for SChatCommandSigned<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_string(self.command)?;
        write.write_i64_be(self.timestamp)?;
        write.write_i64_be(self.salt)?;
        write.write_var_int(&VarInt(self.argument_signatures.len() as i32))?;
        for arg in &self.argument_signatures {
            write.write_string(arg.name)?;
            write.write_slice(arg.signature)?;
        }
        write.write_var_int(&self.message_count)?;
        write.write_slice(self.acknowledged)?;
        if *version >= JavaMinecraftVersion::V_1_21_5 {
            write.write_u8(self.checksum)?;
        }
        Ok(())
    }
}
