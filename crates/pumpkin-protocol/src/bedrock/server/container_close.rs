// Last verified for v2169

use pumpkin_macros::packet;

use crate::serial::{PacketRead, PacketWrite};

/// Sent by either client or server to close an open container inventory window.
#[derive(Debug, PacketWrite, PacketRead)]
#[packet(47)]
pub struct SContainerClose {
    /// Window ID of the container being closed.
    pub container_id: u8,
    /// Bedrock container type identifier.
    pub container_type: u8,
    /// True if the close was initiated by the server rather than the player.
    pub server_initiated_close: bool,
}
