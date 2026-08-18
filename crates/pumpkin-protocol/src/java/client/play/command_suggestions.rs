use crate::ClientPacket;
use crate::VarInt;
use crate::ser::NetworkWriteExt;
use pumpkin_data::packet::clientbound::PLAY_COMMAND_SUGGESTIONS;
use pumpkin_macros::java_packet;
use pumpkin_util::text::TextComponent;
use pumpkin_util::version::JavaMinecraftVersion;

/// Sent by the server to provide a list of "tab-completion" suggestions.
///
/// This packet responds to a client's request for help with commands,
/// appearing as a scrollable list of options while the player is typing
/// in the chat bar.
#[java_packet(PLAY_COMMAND_SUGGESTIONS)]
pub struct CCommandSuggestions {
    /// The unique ID of the request this response is for.
    /// This must match the ID sent by the client in the request packet.
    pub id: VarInt,
    /// The starting character index in the chat bar where the completion
    /// should be inserted.
    pub start: VarInt,
    /// The number of characters in the original text to replace with
    /// the suggestion.
    pub length: VarInt,
    /// The list of possible completions, which can include tooltips
    /// for extra context.
    pub matches: Box<[CommandSuggestion]>,
}

impl CCommandSuggestions {
    #[must_use]
    pub const fn new(
        id: VarInt,
        start: VarInt,
        length: VarInt,
        matches: Box<[CommandSuggestion]>,
    ) -> Self {
        Self {
            id,
            start,
            length,
            matches,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct CommandSuggestion {
    pub suggestion: String,
    pub tooltip: Option<TextComponent>,
}

impl CommandSuggestion {
    #[must_use]
    pub const fn new(suggestion: String, tooltip: Option<TextComponent>) -> Self {
        Self {
            suggestion,
            tooltip,
        }
    }

    pub fn write(&self, mut write: impl std::io::Write) -> Result<(), crate::ser::WritingError> {
        write.write_string(&self.suggestion)?;
        if let Some(tooltip) = &self.tooltip {
            write.write_bool(true)?;
            write.write_slice(&tooltip.encode())?;
        } else {
            write.write_bool(false)?;
        }
        Ok(())
    }
}

impl ClientPacket for CCommandSuggestions {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.id)?;
        write.write_var_int(&self.start)?;
        write.write_var_int(&self.length)?;
        write.write_var_int(&VarInt(self.matches.len() as i32))?;
        for match_ in &self.matches {
            match_.write(&mut write)?;
        }
        Ok(())
    }
}
