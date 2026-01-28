use std::net::SocketAddr;

use pumpkin_macros::packet;

use crate::serial::PacketWrite;
#[derive(PacketWrite)]
#[packet(0x03)]
pub struct CConnectedPong {
    ping: u64,
    pong: u64,
}

impl CConnectedPong {
    #[must_use]
    #[expect(clippy::similar_names)]
    pub fn new(ping: u64, pong: u64) -> Self {
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
    pub fn new(
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
