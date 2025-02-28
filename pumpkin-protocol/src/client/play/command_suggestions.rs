use bytes::BufMut;
use pumpkin_data::packet::clientbound::PLAY_COMMAND_SUGGESTIONS;
use pumpkin_macros::packet;
use pumpkin_util::text::TextComponent;

use crate::{ClientPacket, VarInt, bytebuf::ByteBufMut};

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
    fn write(&self, bytebuf: &mut impl BufMut) {
        bytebuf.put_var_int(&self.id);
        bytebuf.put_var_int(&self.start);
        bytebuf.put_var_int(&self.length);

        bytebuf.put_list(&self.matches, |bytebuf, suggestion| {
            bytebuf.put_string(&suggestion.suggestion);
            bytebuf.put_bool(suggestion.tooltip.is_some());
            if let Some(tooltip) = &suggestion.tooltip {
                bytebuf.put_slice(&tooltip.encode());
            }
        })
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
