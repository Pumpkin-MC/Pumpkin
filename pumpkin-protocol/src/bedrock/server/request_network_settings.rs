use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[packet(0xC1)]
pub struct SRequestNetworkSettings {
    pub protocol_version: i32,
}
