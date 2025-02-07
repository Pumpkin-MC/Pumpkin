use pumpkin_data::packet::serverbound::PLAY_KEEP_ALIVE;
use pumpkin_macros::server_packet;
use serde::Deserialize;

#[derive(Deserialize)]
#[server_packet(PLAY_KEEP_ALIVE)]
pub struct SKeepAlive {
    pub keep_alive_id: i64,
}
