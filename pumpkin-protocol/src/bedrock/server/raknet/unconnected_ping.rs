use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[packet(0x01)]
/// Used to request Server information like MOTD
pub struct SUnconnectedPing {
    pub time: u64,
    pub magic: [u8; 16],
    pub client_guid: u64,
}
