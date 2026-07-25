use std::net::SocketAddr;

use pumpkin_macros::packet;

use crate::{bedrock::RAKNET_MAGIC, serial::PacketWrite};
/// Sent in response to a `ConnectedPing` (`0x00`) to calculate round-trip latency and synchronize time across an established connection.
///
/// Ref: <https://minecraft.wiki/w/RakNet#Connected_Pong>
#[derive(PacketWrite)]
#[packet(0x03)]
pub struct CConnectedPong {
    ping: u64,
    pong: u64,
}

impl CConnectedPong {
    #[must_use]
    #[expect(clippy::similar_names)]
    pub const fn new(ping: u64, pong: u64) -> Self {
        Self { ping, pong }
    }
}

/// Sent by the server to accept an incoming `ConnectionRequest` (`0x09`), confirming connection parameters and system addresses.
///
/// Ref: <https://minecraft.wiki/w/RakNet#Connection_Request_Accepted>
#[derive(PacketWrite)]
#[packet(0x10)]
pub struct CConnectionRequestAccepted {
    client_address: SocketAddr,
    system_index: u16,
    system_addresses: [SocketAddr; 10],
    requested_timestamp: u64,
    timestamp: u64,
}

impl CConnectionRequestAccepted {
    #[must_use]
    pub const fn new(
        client_address: SocketAddr,
        system_index: u16,
        system_addresses: [SocketAddr; 10],
        requested_timestamp: u64,
        timestamp: u64,
    ) -> Self {
        Self {
            client_address,
            system_index,
            system_addresses,
            requested_timestamp,
            timestamp,
        }
    }
}

/// Sent by the server when a client attempts to connect while already being connected.
///
/// Ref: <https://minecraft.wiki/w/RakNet#Already_Connected>
#[derive(PacketWrite)]
#[packet(0x12)]
pub struct CAlreadyConnected {
    magic: [u8; 16],
    server_guid: u64,
}

impl CAlreadyConnected {
    #[must_use]
    pub const fn new(server_guid: u64) -> Self {
        Self {
            magic: RAKNET_MAGIC,
            server_guid,
        }
    }
}

/// Sent by the server when it has reached its maximum connection capacity.
///
/// Ref: <https://minecraft.wiki/w/RakNet#No_Free_Incoming_Connections>
#[derive(PacketWrite)]
#[packet(0x14)]
pub struct CNoFreeIncomingConnections {
    magic: [u8; 16],
    server_guid: u64,
}

impl CNoFreeIncomingConnections {
    #[must_use]
    pub const fn new(server_guid: u64) -> Self {
        Self {
            magic: RAKNET_MAGIC,
            server_guid,
        }
    }
}

/// Sent by the server when a client attempts to connect from a banned IP address or identifier.
///
/// Ref: <https://minecraft.wiki/w/RakNet#Connection_Banned>
#[derive(PacketWrite)]
#[packet(0x17)]
pub struct CConnectionBanned {
    magic: [u8; 16],
    server_guid: u64,
}

impl CConnectionBanned {
    #[must_use]
    pub const fn new(server_guid: u64) -> Self {
        Self {
            magic: RAKNET_MAGIC,
            server_guid,
        }
    }
}

/// Sent by the server when a client attempts to connect again too quickly after disconnecting.
///
/// Ref: <https://minecraft.wiki/w/RakNet#IP_Recently_Connected>
#[derive(PacketWrite)]
#[packet(0x1A)]
pub struct CIpRecentlyConnected {
    magic: [u8; 16],
    server_guid: u64,
}

impl CIpRecentlyConnected {
    #[must_use]
    pub const fn new(server_guid: u64) -> Self {
        Self {
            magic: RAKNET_MAGIC,
            server_guid,
        }
    }
}

/// Sent by the server to initiate graceful termination of the connection session.
///
/// Ref: <https://minecraft.wiki/w/RakNet#Disconnection_Notification>
#[derive(PacketWrite)]
#[packet(0x15)]
pub struct CDisconnect;
