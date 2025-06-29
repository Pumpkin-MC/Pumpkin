use pumpkin_data::packet::serverbound::PLAY_CHAT_COMMAND;
use pumpkin_macros::packet;
use serde::Serialize;

#[derive(serde::Deserialize, Serialize)]
#[packet(PLAY_CHAT_COMMAND)]
pub struct SChatCommand {
    pub command: String,
}
