use std::{borrow::Cow, io::Write};

use pumpkin_data::packet::clientbound::PLAY_COMMAND_SUGGESTIONS;
use pumpkin_macros::packet;
use pumpkin_util::text::TextComponent;

use crate::{
    ClientPacket, VarInt,
    ser::{NetworkWriteExt, WritingError},
};

#[packet(PLAY_COMMAND_SUGGESTIONS)]
pub struct CCommandSuggestions<'a> {
    id: VarInt,
    start: VarInt,
    length: VarInt,
    matches: Vec<CommandSuggestion<'a>>,
}

impl<'a> CCommandSuggestions<'a> {
    pub fn new(
        id: VarInt,
        start: VarInt,
        length: VarInt,
        matches: Vec<CommandSuggestion<'a>>,
    ) -> Self {
        Self {
            id,
            start,
            length,
            matches,
        }
    }
}

impl ClientPacket for CCommandSuggestions<'_> {
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
pub struct CommandSuggestion<'a> {
    pub suggestion: Cow<'a, str>,
    pub tooltip: Option<TextComponent>,
}

impl<'a> CommandSuggestion<'a> {
    pub fn new(suggestion: Cow<'a, str>, tooltip: Option<TextComponent>) -> Self {
        Self {
            suggestion,
            tooltip,
        }
    }
}
