use pumpkin_data::packet::clientbound::STATUS_PONG_RESPONSE;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[packet(STATUS_PONG_RESPONSE)]
pub struct CPingResponse {
    payload: i64, // must respond with the same as in `SPingRequest`
}

impl CPingResponse {
    pub fn new(payload: i64) -> Self {
        Self { payload }
    }
}
