use pumpkin_data::packet::serverbound::PLAY_CHANGE_GAME_MODE;
use pumpkin_macros::packet;
use serde::Deserialize;

use crate::VarInt;

#[derive(Deserialize)]
#[packet(PLAY_CHANGE_GAME_MODE)]
pub struct SChangeGameMode {
    pub gamemode: VarInt,
}
