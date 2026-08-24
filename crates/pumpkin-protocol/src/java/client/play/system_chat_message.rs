use pumpkin_data::packet::clientbound::play::{CHAT, SYSTEM_CHAT};
use pumpkin_util::text::TextComponent;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::ClientPacket;
use crate::codec::var_int::VarInt;
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
    fn to_id(version: JavaMinecraftVersion) -> i32 {
        if version >= JavaMinecraftVersion::V_1_19 {
            SYSTEM_CHAT.to_id(version)
        } else {
            CHAT.to_id(version)
        }
    }
}

impl ClientPacket for CSystemChatMessage<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_component(self.content, version)?;

        if *version >= JavaMinecraftVersion::V_1_19_1 {
            write.write_bool(self.overlay)?;
        } else if *version >= JavaMinecraftVersion::V_1_19 {
            // In 1.19.0, type ID was a VarInt: 1 for SYSTEM, 2 for GAME_INFO (actionbar/overlay)
            let type_id = if self.overlay { 2 } else { 1 };
            write.write_var_int(&VarInt(type_id))?;
        } else if *version >= JavaMinecraftVersion::V_1_16 {
            // In 1.16 - 1.18.2: position byte (1 for system, 2 for game_info) + sender UUID
            let position = if self.overlay { 2 } else { 1 };
            write.write_u8(position)?;
            write.write_uuid(&uuid::Uuid::nil())?;
        } else if *version >= JavaMinecraftVersion::V_1_8 {
            // In 1.8 - 1.15.2: position byte (1 for system, 2 for game_info)
            let position = if self.overlay { 2 } else { 1 };
            write.write_u8(position)?;
        }
        // In 1.7.2 - 1.7.10: only component was present in the chat packet

        Ok(())
    }
}
