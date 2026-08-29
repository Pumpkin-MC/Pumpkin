// Last verified for v2169

use pumpkin_macros::packet;

use crate::serial::{PacketRead, PacketWrite};

#[derive(Debug, PacketWrite, PacketRead)]
#[packet(47)]
pub struct SContainerClose {
    pub container_id: u8,
    pub container_type: u8,
    pub server_initiated_close: bool,
}
