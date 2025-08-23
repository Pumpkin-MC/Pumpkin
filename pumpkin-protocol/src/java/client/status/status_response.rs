use pumpkin_data::packet::clientbound::STATUS_STATUS_RESPONSE;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[packet(STATUS_STATUS_RESPONSE)]
pub struct CStatusResponse {
    pub json_response: String, // 32767
}
impl CStatusResponse {
    pub fn new(json_response: String) -> Self {
        Self { json_response }
    }
}
