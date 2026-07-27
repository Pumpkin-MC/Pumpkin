use std::net::SocketAddr;

use pumpkin_macros::packet;

use crate::{bedrock::RAKNET_MAGIC, serial::PacketWrite};

/// Sent by the server in response to `OpenConnectionRequest1` (`0x05`), negotiating security options, server GUID, and MTU.
///
/// Ref: <https://minecraft.wiki/w/RakNet#Open_Connection_Reply_1>
#[derive(PacketWrite)]
#[packet(0x06)]
pub struct COpenConnectionReply1 {
    magic: [u8; 16],
    server_guid: u64,
    has_server_security: bool,
    // Only write when has_server_security
    // cookie: u32,
    mtu: u16,
}

impl COpenConnectionReply1 {
    #[must_use]
    pub const fn new(server_guid: u64, has_server_security: bool, mtu: u16) -> Self {
        Self {
            magic: RAKNET_MAGIC,
            server_guid,
            has_server_security,
            // cookie,
            mtu,
        }
    }
}

/// Sent by the server in response to `OpenConnectionRequest2` (`0x07`), confirming the connection setup and client address before establishing session state.
///
/// Ref: <https://minecraft.wiki/w/RakNet#Open_Connection_Reply_2>
#[derive(PacketWrite)]
#[packet(0x08)]
pub struct COpenConnectionReply2 {
    magic: [u8; 16],
    server_guid: u64,
    client_address: SocketAddr,
    mtu: u16,
    security: bool,
}

impl COpenConnectionReply2 {
    #[must_use]
    pub const fn new(
        server_guid: u64,
        client_address: SocketAddr,
        mtu: u16,
        security: bool,
    ) -> Self {
        Self {
            magic: RAKNET_MAGIC,
            server_guid,
            client_address,
            mtu,
            security,
        }
    }
}
