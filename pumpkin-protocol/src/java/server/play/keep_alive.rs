use pumpkin_data::packet::serverbound::PLAY_KEEP_ALIVE;
use pumpkin_macros::java_packet;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[java_packet(PLAY_KEEP_ALIVE)]
pub struct SKeepAlive {
    pub keep_alive_id: i64,
}
