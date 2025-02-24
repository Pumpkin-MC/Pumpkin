use pumpkin_data::packet::clientbound::STATUS_STATUS_RESPONSE;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[packet(STATUS_STATUS_RESPONSE)]
pub struct CStatusResponse<'a> {
    json_response: &'a str, // 32767
}
impl<'a> CStatusResponse<'a> {
    pub fn new(json_response: &'a str) -> Self {
        Self { json_response }
    }
}
