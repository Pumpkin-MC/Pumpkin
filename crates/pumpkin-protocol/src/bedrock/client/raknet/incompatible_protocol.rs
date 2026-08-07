use crate::{bedrock::RAKNET_MAGIC, serial::PacketWrite};
use pumpkin_macros::packet;

/// Sent by the server when the client's `RakNet` protocol version does not match the server's expected protocol version (`11`).
///
/// Ref: <https://minecraft.wiki/w/RakNet#Incompatible_Protocol_Version>
#[derive(PacketWrite)]
#[packet(0x19)]
pub struct CIncompatibleProtocolVersion {
    protocol_version: u8,
    magic: [u8; 16],
    server_guid: u64,
}

impl CIncompatibleProtocolVersion {
    #[must_use]
    pub const fn new(protocol_version: u8, server_guid: u64) -> Self {
        Self {
            protocol_version,
            magic: RAKNET_MAGIC,
            server_guid,
        }
    }
}
