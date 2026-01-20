use std::net::SocketAddr;

use pumpkin_macros::packet;

use crate::serial::PacketWrite;
#[derive(PacketWrite)]
#[packet(0x03)]
pub struct CConnectedPong {
    ping_time: u64,
    pong_time: u64,
}

impl CConnectedPong {
    #[must_use] 
    pub const fn new(ping_time: u64, pong_time: u64) -> Self {
        Self {
            ping_time,
            pong_time,
        }
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
