use pumpkin_data::packet::serverbound::PLAY_CHANGE_GAME_MODE;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

use crate::VarInt;

#[derive(Deserialize, Serialize)]
#[packet(PLAY_CHANGE_GAME_MODE)]
pub struct SChangeGameMode {
    /// 0: survival, 1: creative, 2: adventure, 3: spectator
    pub game_mode: VarInt,
}
