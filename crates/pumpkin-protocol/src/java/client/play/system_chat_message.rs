use pumpkin_data::packet::clientbound::play::SYSTEM_CHAT;
use pumpkin_util::text::TextComponent;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::ClientPacket;
use crate::packet::MultiVersionJavaPacket;
use crate::ser::NetworkWriteExt;

/// Sends a system chat message to the client.
///
/// System messages are messages sent by the server itself (such as join/quit notices,
/// command feedback, server announcements, or actionbar overlay messages).
pub struct CSystemChatMessage<'a> {
    pub content: &'a TextComponent,
    /// When true, the message is displayed above the hotbar (actionbar).
    /// When false, it is displayed in the normal chat box.
    pub overlay: bool,
}

impl<'a> CSystemChatMessage<'a> {
    #[must_use]
    pub const fn new(content: &'a TextComponent, overlay: bool) -> Self {
        Self { content, overlay }
    }
}

impl MultiVersionJavaPacket for CSystemChatMessage<'_> {
    fn to_id(_version: JavaMinecraftVersion) -> i32 {
        SYSTEM_CHAT.to_id()
    }
}

impl ClientPacket for CSystemChatMessage<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_component(self.content)?;

        write.write_bool(self.overlay)?;
        // In 1.7.2 - 1.7.10: only component was present in the chat packet

        Ok(())
    }
}
