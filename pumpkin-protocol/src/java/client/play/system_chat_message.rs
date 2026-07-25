use pumpkin_data::packet::clientbound::PLAY_SYSTEM_CHAT;
use pumpkin_util::text::TextComponent;

use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_SYSTEM_CHAT)]
pub struct CSystemChatMessage<'a> {
    pub content: &'a TextComponent,
    pub overlay: bool,
}

impl<'a> CSystemChatMessage<'a> {
    #[must_use]
    pub const fn new(content: &'a TextComponent, overlay: bool) -> Self {
        Self { content, overlay }
    }
}

impl ClientPacket for CSystemChatMessage<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_slice(&self.content.encode())?;
        write.write_bool(self.overlay)?;
        Ok(())
    }
}
