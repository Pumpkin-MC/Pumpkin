use pumpkin_data::packet::clientbound::PLAY_PONG_RESPONSE;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[packet(PLAY_PONG_RESPONSE)]
pub struct CPingResponse {
    pub payload: i64,
}

impl CPingResponse {
    pub fn new(payload: i64) -> Self {
        Self { payload }
    }
}
