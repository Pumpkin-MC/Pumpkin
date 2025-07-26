use std::net::SocketAddr;

use pumpkin_macros::packet;

use crate::serial::PacketRead;

#[derive(PacketRead)]
#[packet(0x00)]
pub struct SConnectedPing {
    /// Time since start
    #[serial(big_endian)]
    pub time: u64,
}

#[derive(PacketRead)]
#[packet(0x09)]
pub struct SConnectionRequest {
    #[serial(big_endian)]
    pub client_guid: u64,
    #[serial(big_endian)]
    pub time: u64,
    pub security: bool,
}

#[derive(PacketRead)]
#[packet(0x13)]
pub struct SNewIncomingConnection {
    pub server_address: SocketAddr,
    pub internal_address: SocketAddr,
    #[serial(big_endian)]
    pub ping_time: u64,
    #[serial(big_endian)]
    pub pong_time: u64,
}

#[packet(0x15)]
pub struct SDisconnect;
