use pumpkin_data::packet::clientbound::PLAY_PONG_RESPONSE;
use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet(PLAY_PONG_RESPONSE)]
pub struct CPingResponse {
    payload: i64,
}

impl CPingResponse {
    pub fn new(payload: i64) -> Self {
        Self { payload }
    }
}
