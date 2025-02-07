use pumpkin_data::packet::serverbound::PLAY_COMMAND_SUGGESTION;
use pumpkin_macros::server_packet;
use serde::Deserialize;

use crate::VarInt;

#[derive(Deserialize)]
#[server_packet(PLAY_COMMAND_SUGGESTION)]
pub struct SCommandSuggestion {
    pub id: VarInt,
    pub command: String,
}
