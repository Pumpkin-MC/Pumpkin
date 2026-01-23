use pumpkin_data::packet::clientbound::PLAY_COMMAND_SUGGESTIONS;
use pumpkin_macros::packet;
use pumpkin_util::text::TextComponent;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[packet(PLAY_COMMAND_SUGGESTIONS)]
pub struct CCommandSuggestions {
    pub id: VarInt,
    pub start: VarInt,
    pub length: VarInt,
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
