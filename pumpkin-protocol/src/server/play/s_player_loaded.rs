use pumpkin_data::packet::serverbound::PLAY_PLAYER_LOADED;
use pumpkin_macros::server_packet;

#[derive(serde::Deserialize)]
#[server_packet(PLAY_PLAYER_LOADED)]
pub struct SPlayerLoaded;
