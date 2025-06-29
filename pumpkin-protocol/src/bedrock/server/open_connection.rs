use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

use crate::codec::socket_address::SocketAddress;

#[derive(Serialize, Deserialize)]
#[packet(0x05)]
/// The client sends this when attempting to join the server
pub struct SOpenConnectionRequest1 {
    pub magic: [u8; 16],
    pub protocol_version: u8,
    pub mtu: u16,
}

#[derive(Serialize, Deserialize)]
#[packet(0x07)]
pub struct SOpenConnectionRequest2 {
    pub magic: [u8; 16],
    pub server_address: SocketAddress,
    pub mtu: u16,
    pub client_guid: u64,
}
