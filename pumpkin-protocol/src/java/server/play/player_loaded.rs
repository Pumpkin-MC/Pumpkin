use pumpkin_data::packet::serverbound::PLAY_PLAYER_LOADED;
use pumpkin_macros::packet;

#[packet(PLAY_PLAYER_LOADED)]
pub struct SPlayerLoaded;
