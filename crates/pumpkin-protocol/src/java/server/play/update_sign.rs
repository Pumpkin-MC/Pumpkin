use pumpkin_data::packet::serverbound::play::SIGN_UPDATE;
use pumpkin_macros::java_packet;
use pumpkin_util::{math::position::BlockPos, version::JavaMinecraftVersion};

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, NetworkReadSliceExt, ReadingError},
};

#[java_packet(SIGN_UPDATE)]
pub struct SUpdateSign<'a> {
    pub location: BlockPos,
    pub is_front_text: bool,
    pub line_1: &'a str,
    pub line_2: &'a str,
    pub line_3: &'a str,
    pub line_4: &'a str,
}

const MAX_LINE_LENGTH: usize = 384;

impl<'a> ServerPacket<'a> for SUpdateSign<'a> {
    fn read(read: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let location = read.get_block_pos(version)?;
        let is_front_text = if *version >= JavaMinecraftVersion::V_1_20 {
            read.get_bool()?
        } else {
            true
        };
        let line_1 = read.get_str_bounded_borrowed(MAX_LINE_LENGTH)?;
        let line_2 = read.get_str_bounded_borrowed(MAX_LINE_LENGTH)?;
        let line_3 = read.get_str_bounded_borrowed(MAX_LINE_LENGTH)?;
        let line_4 = read.get_str_bounded_borrowed(MAX_LINE_LENGTH)?;

        Ok(Self {
            location,
            is_front_text,
            line_1,
            line_2,
            line_3,
            line_4,
        })
    }
}

impl crate::ClientPacket for SUpdateSign<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_block_pos(&self.location, version)?;
        if *version >= JavaMinecraftVersion::V_1_20 {
            write.write_bool(self.is_front_text)?;
        }
        write.write_string_bounded(self.line_1, MAX_LINE_LENGTH)?;
        write.write_string_bounded(self.line_2, MAX_LINE_LENGTH)?;
        write.write_string_bounded(self.line_3, MAX_LINE_LENGTH)?;
        write.write_string_bounded(self.line_4, MAX_LINE_LENGTH)?;
        Ok(())
    }
}
