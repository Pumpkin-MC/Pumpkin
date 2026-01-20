use pumpkin_data::packet::clientbound::STATUS_PONG_RESPONSE;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[packet(STATUS_PONG_RESPONSE)]
pub struct CPingResponse {
    pub payload: i64, // must respond with the same as in `SPingRequest`
}

impl CPingResponse {
    #[must_use] 
    pub const fn new(payload: i64) -> Self {
        Self { payload }
    }
}
