use std::net::SocketAddr;

use pumpkin_macros::packet;

use crate::{bedrock::RAKNET_MAGIC, serial::PacketWrite};
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

#[derive(PacketWrite)]
#[packet(0x15)]
pub struct CDisconnect;
