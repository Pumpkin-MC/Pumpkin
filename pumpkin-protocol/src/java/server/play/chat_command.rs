use pumpkin_data::packet::serverbound::PLAY_CHAT_COMMAND;
use pumpkin_macros::java_packet;
use serde::Serialize;

#[derive(serde::Deserialize, Serialize)]
#[java_packet(PLAY_CHAT_COMMAND)]
pub struct SChatCommand {
    pub command: String,
}
