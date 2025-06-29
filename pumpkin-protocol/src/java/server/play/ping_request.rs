use pumpkin_data::packet::serverbound::PLAY_PING_REQUEST;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[packet(PLAY_PING_REQUEST)]
pub struct SPlayPingRequest {
    pub payload: i64,
}
