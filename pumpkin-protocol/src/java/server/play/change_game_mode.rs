use pumpkin_data::packet::serverbound::PLAY_CHANGE_GAME_MODE;
use pumpkin_macros::packet;
use pumpkin_util::GameMode;
use serde::Serialize;

#[derive(serde::Deserialize, Serialize)]
#[packet(PLAY_CHANGE_GAME_MODE)]
pub struct SChangeGameMode {
    pub game_mode: GameMode,
}
