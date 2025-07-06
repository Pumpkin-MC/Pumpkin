use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[packet(0x01)]
pub struct SLogin {
    pub protocol_version: i32,
    // TODO: Add More
}
