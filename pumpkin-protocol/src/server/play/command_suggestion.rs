use pumpkin_data::packet::serverbound::PLAY_COMMAND_SUGGESTION;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

use crate::VarInt;

#[derive(Serialize, Deserialize)]
#[packet(PLAY_COMMAND_SUGGESTION)]
pub struct SCommandSuggestion {
    pub id: VarInt,
    pub command: String,
}
