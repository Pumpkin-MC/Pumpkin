use pumpkin_data::packet::clientbound::PLAY_COMMAND_SUGGESTIONS;
use pumpkin_macros::java_packet;
use pumpkin_util::text::TextComponent;
use serde::Serialize;

use crate::VarInt;

/// Sent by the server to provide a list of "tab-completion" suggestions.
///
/// This packet responds to a client's request for help with commands,
/// appearing as a scrollable list of options while the player is typing
/// in the chat bar.
#[derive(Serialize)]
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

#[derive(PartialEq, Eq, Hash, Debug, Serialize)]
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
}
