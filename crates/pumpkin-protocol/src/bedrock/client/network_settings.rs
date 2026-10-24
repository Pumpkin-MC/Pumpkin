// Last verified for v2169

use pumpkin_macros::packet;

use crate::serial::PacketWrite;

/// Sent by the server in response to [`SRequestNetworkSettings`](crate::bedrock::server::request_network_settings::SRequestNetworkSettings) to configure packet compression and client throttling.
#[derive(PacketWrite)]
#[packet(143)]
pub struct CNetworkSettings {
    /// Byte size threshold above which packet payloads are compressed. If zero, packets are uncompressed.
    pub compression_threshold: u16,

    // TODO: CompressionAlgorithm enum
    /// The compression algorithm to use (`0` = Deflate/ZLib, `1` = Snappy, `0xffff` = None).
    pub compression_algorithm: u16,

    /// Whether the client should throttle player ticking when player count exceeds threshold.
    pub client_throttle_enabled: bool,
    /// Player count limit before the client activates throttling.
    pub client_throttle_threshold: u8,
    /// Fraction of players ticked per frame when throttling is engaged.
    pub client_throttle_scalar: f32,
}
