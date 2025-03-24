use std::io::Write;

use pumpkin_data::packet::clientbound::PLAY_COMMAND_SUGGESTIONS;
use pumpkin_macros::packet;
use pumpkin_util::text::TextComponent;

use crate::{
    ClientPacket, VarInt,
    ser::{NetworkWriteExt, WritingError},
};

#[packet(PLAY_COMMAND_SUGGESTIONS)]
pub struct CCommandSuggestions {
    id: VarInt,
    start: VarInt,
    length: VarInt,
    matches: Vec<CommandSuggestion>,
}

impl CCommandSuggestions {
    pub fn new(id: VarInt, start: VarInt, length: VarInt, matches: Vec<CommandSuggestion>) -> Self {
        Self {
            id,
            start,
            length,
            matches,
        }
    }
}

impl ClientPacket for CCommandSuggestions {
    fn write_packet_data(&self, write: impl Write) -> Result<(), WritingError> {
        let mut write = write;
        write.write_var_int(&self.id)?;
        write.write_var_int(&self.start)?;
        write.write_var_int(&self.length)?;

        write.write_list(&self.matches, |write, suggestion| {
            write.write_string(&suggestion.suggestion)?;
            write.write_bool(suggestion.tooltip.is_some())?;
            if let Some(tooltip) = &suggestion.tooltip {
                write.write_slice(&tooltip.encode())?;
            }

            Ok(())
        })?;

        Ok(())
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct CommandSuggestion {
    pub suggestion: String,
    pub tooltip: Option<TextComponent>,
}

impl CommandSuggestion {
    pub fn new(suggestion: String, tooltip: Option<TextComponent>) -> Self {
        Self {
            suggestion,
            tooltip,
        }
    }
}
