use pumpkin_data::packet::serverbound::PLAY_PLAYER_INPUT;
use pumpkin_macros::packet;

#[derive(serde::Deserialize)]
#[packet(PLAY_PLAYER_INPUT)]
pub struct SPlayerInput {
    // Yep exactly how it looks like
    _input: i8,
}
