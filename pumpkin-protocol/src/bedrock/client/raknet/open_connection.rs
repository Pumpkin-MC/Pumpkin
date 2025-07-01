use pumpkin_macros::packet;
use serde::Serialize;

use crate::{bedrock::RAKNET_MAGIC, codec::socket_address::SocketAddress};

#[derive(Serialize)]
#[packet(0x06)]
pub struct COpenConnectionReply1 {
    magic: [u8; 16],
    server_guid: u64,
    has_server_security: bool,
    cookie: u32,
    mtu: u16,
}

impl COpenConnectionReply1 {
    pub fn new(server_guid: u64, has_server_security: bool, cookie: u32, mtu: u16) -> Self {
        Self {
            magic: RAKNET_MAGIC,
            server_guid,
            has_server_security,
            cookie,
            mtu,
        }
    }
}

#[derive(Serialize)]
#[packet(0x08)]
pub struct COpenConnectionReply2 {
    magic: [u8; 16],
    server_guid: u64,
    client_address: SocketAddress,
    mtu: u16,
    security: bool,
}

impl COpenConnectionReply2 {
    pub fn new(server_guid: u64, client_address: SocketAddress, mtu: u16, security: bool) -> Self {
        Self {
            magic: RAKNET_MAGIC,
            server_guid,
            client_address,
            mtu,
            security,
        }
    }
}
