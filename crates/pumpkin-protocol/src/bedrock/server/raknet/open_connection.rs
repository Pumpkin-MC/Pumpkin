use std::net::SocketAddr;

use pumpkin_macros::packet;

use crate::serial::PacketRead;

/// Sent by a connecting client to initiate `RakNet` handshake and check server MTU size.
///
/// Ref: <https://minecraft.wiki/w/RakNet#Open_Connection_Request_1>
#[derive(PacketRead)]
#[packet(0x05)]
pub struct SOpenConnectionRequest1 {
    pub magic: [u8; 16],
    pub protocol_version: u8,
    #[serial(big_endian)]
    pub mtu: u16,
}

/// Sent by a connecting client following `OpenConnectionReply1` to verify server address, client GUID, and MTU.
///
/// Ref: <https://minecraft.wiki/w/RakNet#Open_Connection_Request_2>
#[derive(PacketRead)]
#[packet(0x07)]
pub struct SOpenConnectionRequest2 {
    pub magic: [u8; 16],
    pub server_address: SocketAddr,
    #[serial(big_endian)]
    pub mtu: u16,
    #[serial(big_endian)]
    pub client_guid: u64,
}
