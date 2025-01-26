use pumpkin_data::packets::serverbound::PLAY_CHAT_COMMAND;
use pumpkin_macros::server_packet;

#[derive(serde::Deserialize)]
#[server_packet(PLAY_CHAT_COMMAND)]
pub struct SChatCommand {
    pub command: String,
}
