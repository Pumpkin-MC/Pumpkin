use pumpkin_data::packet::clientbound::PLAY_KEEP_ALIVE;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[packet(PLAY_KEEP_ALIVE)]
pub struct CKeepAlive {
    pub keep_alive_id: i64,
}

impl CKeepAlive {
    pub fn new(keep_alive_id: i64) -> Self {
        Self { keep_alive_id }
    }
}
