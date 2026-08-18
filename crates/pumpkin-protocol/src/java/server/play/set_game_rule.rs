use pumpkin_data::packet::serverbound::PLAY_SET_GAME_RULE;
use pumpkin_macros::java_packet;

use crate::{
    ServerPacket,
    ser::{NetworkReadSliceExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_SET_GAME_RULE)]
pub struct SSetGameRule<'a> {
    pub rule: &'a str,
    pub value: &'a str,
}

impl<'a> ServerPacket<'a> for SSetGameRule<'a> {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            rule: bytebuf.get_str_borrowed()?,
            value: bytebuf.get_str_borrowed()?,
        })
    }
}

impl crate::ClientPacket for SSetGameRule<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_string(self.rule)?;
        write.write_string(self.value)?;
        Ok(())
    }
}
