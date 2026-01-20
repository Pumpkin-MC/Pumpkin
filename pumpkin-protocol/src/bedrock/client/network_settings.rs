use pumpkin_macros::packet;

use crate::serial::PacketWrite;

#[derive(PacketWrite)]
#[packet(143)]
pub struct CNetworkSettings {
    compression_threshold: u16,
    compression_method: u16,
    client_throttle_enabled: bool,
    client_throttle_threshold: i8,
    client_throttle_scalar: f32,
}

impl CNetworkSettings {
    #[must_use] 
    pub const fn new(
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
