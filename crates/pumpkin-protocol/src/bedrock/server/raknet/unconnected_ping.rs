use pumpkin_macros::packet;

use crate::serial::PacketRead;

/// Sent by an unconnected client to request server information, status, and MOTD.
///
/// Ref: <https://minecraft.wiki/w/RakNet#Unconnected_Ping>
#[derive(PacketRead)]
#[packet(0x01)]
pub struct SUnconnectedPing {
    #[serial(big_endian)]
    pub time: u64,
    pub magic: [u8; 16],
    #[serial(big_endian)]
    pub client_guid: u64,
}

/// Sent by a client to query server information when connections are open.
///
/// Ref: <https://minecraft.wiki/w/RakNet#Unconnected_Ping_Open_Connections>
#[derive(PacketRead)]
#[packet(0x02)]
pub struct SUnconnectedPingOpenConnections {
    #[serial(big_endian)]
    pub time: u64,
    pub magic: [u8; 16],
    #[serial(big_endian)]
    pub client_guid: u64,
}
