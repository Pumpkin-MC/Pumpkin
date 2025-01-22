use pumpkin_data::packet::clientbound::PLAY_KEEP_ALIVE;
use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet(PLAY_KEEP_ALIVE)]
pub struct CKeepAlive {
    keep_alive_id: i64,
}

impl CKeepAlive {
    pub fn new(keep_alive_id: i64) -> Self {
        Self { keep_alive_id }
    }
}
