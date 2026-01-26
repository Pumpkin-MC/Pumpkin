use pumpkin_data::packet::clientbound::LOGIN_LOGIN_COMPRESSION;
use pumpkin_macros::java_packet;
use serde::{Deserialize, Serialize};

use crate::VarInt;

/// Sent by the server to enable network compression for all subsequent packets.
///
/// Once this packet is sent, both the server and the client must compress any
/// packet with a size equal to or greater than the specified threshold.
#[derive(Serialize, Deserialize)]
#[java_packet(LOGIN_LOGIN_COMPRESSION)]
pub struct CSetCompression {
    /// The packet size threshold (in bytes) at which compression is applied.
    ///
    /// Packets smaller than this are sent uncompressed. A negative threshold
    /// typically disables compression.
    pub threshold: VarInt,
}

impl CSetCompression {
    pub fn new(threshold: VarInt) -> Self {
        Self { threshold }
    }
}
