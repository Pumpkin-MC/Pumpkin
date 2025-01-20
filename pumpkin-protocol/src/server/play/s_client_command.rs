use pumpkin_data::packet::serverbound::PLAY_CLIENT_COMMAND;
use pumpkin_macros::server_packet;
use serde::Deserialize;

use crate::VarInt;

#[derive(Deserialize)]
#[server_packet(PLAY_CLIENT_COMMAND)]
pub struct SClientCommand {
    pub action_id: VarInt,
}
