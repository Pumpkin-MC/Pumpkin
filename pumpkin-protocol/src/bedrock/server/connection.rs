use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[packet(0x09)]
pub struct SConnectionRequest {
    pub client_guid: u64,
    pub time: u64,
    pub security: bool,
}
