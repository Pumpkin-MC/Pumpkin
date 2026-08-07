use std::net::SocketAddr;

use pumpkin_macros::packet;

use crate::serial::PacketRead;

/// Sent periodically by a connected client to measure round-trip time.
///
/// Ref: <https://minecraft.wiki/w/RakNet#Connected_Ping>
#[derive(PacketRead)]
#[packet(0x00)]
pub struct SConnectedPing {
    /// Time since start
    #[serial(big_endian)]
    pub time: u64,
}

/// Sent by the client after receiving `OpenConnectionReply2` to request formal session establishment.
///
/// Ref: <https://minecraft.wiki/w/RakNet#Connection_Request>
#[derive(PacketRead)]
#[packet(0x09)]
pub struct SConnectionRequest {
    #[serial(big_endian)]
    pub client_guid: u64,
    #[serial(big_endian)]
    pub time: u64,
    pub security: bool,
}

/// Sent by the client to confirm local network address and finish connection establishment.
///
/// Ref: <https://minecraft.wiki/w/RakNet#New_Incoming_Connection>
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

/// Sent by the client to notify the server of graceful disconnection.
///
/// Ref: <https://minecraft.wiki/w/RakNet#Disconnection_Notification>
#[packet(0x15)]
pub struct SDisconnect;

/// Internal notification signal for a connection lost due to socket error or timeout.
///
/// Ref: <https://minecraft.wiki/w/RakNet#Disconnection_Notification>
#[packet(0x16)]
pub struct SConnectionLost;
