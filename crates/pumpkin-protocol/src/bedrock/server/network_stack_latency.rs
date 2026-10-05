// Last verified for v2208

use pumpkin_macros::packet;

use crate::serial::PacketRead;

#[derive(Debug, PacketRead)]
#[packet(115)]
pub struct SNetworkStackLatency {
    pub timestamp: u64,
    pub from_server: bool,
}
