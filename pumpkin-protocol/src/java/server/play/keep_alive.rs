use pumpkin_data::packet::serverbound::PLAY_KEEP_ALIVE;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[packet(PLAY_KEEP_ALIVE)]
pub struct SKeepAlive {
    pub keep_alive_id: i64,
}
