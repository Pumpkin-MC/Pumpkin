use pumpkin_data::packet::clientbound::PLAY_CUSTOM_CHAT_COMPLETIONS;
use pumpkin_macros::java_packet;

use crate::{ClientPacket, codec::var_int::VarInt, ser::NetworkWriteExt};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_CUSTOM_CHAT_COMPLETIONS)]
pub struct CCustomChatCompletions<'a> {
    pub action: VarInt,
    pub entries: &'a [&'a str],
}

impl<'a> CCustomChatCompletions<'a> {
    #[must_use]
    pub const fn new(action: VarInt, entries: &'a [&'a str]) -> Self {
        Self { action, entries }
    }
}

impl ClientPacket for CCustomChatCompletions<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.action)?;
        write.write_var_int(&VarInt(self.entries.len() as i32))?;
        for entry in self.entries {
            write.write_string(entry)?;
        }
        Ok(())
    }
}
