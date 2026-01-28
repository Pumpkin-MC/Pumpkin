use pumpkin_data::packet::serverbound::PLAY_PLAYER_LOADED;
use pumpkin_macros::java_packet;

#[java_packet(PLAY_PLAYER_LOADED)]
pub struct SPlayerLoaded;
