// Last verified for v2208

use pumpkin_macros::packet;

use crate::serial::PacketWrite;

/// Ping the client echoes back in order. Used to know when earlier packets were applied.
#[derive(PacketWrite)]
#[packet(115)]
pub struct CNetworkStackLatency {
    pub timestamp: u64,
    pub needs_response: bool,
}
