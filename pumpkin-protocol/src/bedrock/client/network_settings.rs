use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[packet(0x8F)]
pub struct CNetworkSettings {
    compression_threshold: u16,
    compression_method: u16,
    client_throttle_enabled: bool,
    client_throttle_threshold: i8,
    client_throttle_scalar: f32,
}

impl CNetworkSettings {
    pub fn new(
        compression_threshold: u16,
        compression_method: u16,
        client_throttle_enabled: bool,
        client_throttle_threshold: i8,
        client_throttle_scalar: f32,
    ) -> Self {
        Self {
            compression_threshold,
            compression_method,
            client_throttle_enabled,
            client_throttle_threshold,
            client_throttle_scalar,
        }
    }
}
