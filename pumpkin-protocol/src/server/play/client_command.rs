use pumpkin_data::packet::serverbound::PLAY_CLIENT_COMMAND;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

use crate::VarInt;

#[derive(Serialize, Deserialize)]
#[packet(PLAY_CLIENT_COMMAND)]
pub struct SClientCommand {
    pub action_id: VarInt,
}
