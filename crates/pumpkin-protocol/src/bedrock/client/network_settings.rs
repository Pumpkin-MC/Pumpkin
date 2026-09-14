// Last verified for v2169

use pumpkin_macros::packet;

use crate::serial::PacketWrite;

#[derive(PacketWrite)]
#[packet(143)]
pub struct CNetworkSettings {
    pub compression_threshold: u16,

    // TODO: CompressionAlgorithm enum
    /// `ZLib` = 0, Snappy = 1, None = 255
    pub compression_algorithm: u16,

    pub client_throttle_enabled: bool,
    pub client_throttle_threshold: u8,
    pub client_throttle_scalar: f32,
}
