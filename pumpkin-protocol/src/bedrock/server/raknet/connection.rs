use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

use crate::codec::socket_address::SocketAddress;

#[derive(Serialize, Deserialize)]
#[packet(0x09)]
pub struct SConnectionRequest {
    pub client_guid: u64,
    pub time: u64,
    pub security: bool,
}

#[derive(Serialize, Deserialize)]
#[packet(0x13)]
pub struct SNewIncomingConnection {
    pub server_address: SocketAddress,
    pub internal_address: SocketAddress,
    pub ping_time: u64,
    pub pong_time: u64,
}

#[derive(Serialize, Deserialize)]
#[packet(0x15)]
pub struct SDisconnect;
