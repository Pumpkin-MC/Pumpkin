use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

use crate::codec::socket_address::SocketAddress;

#[derive(Serialize, Deserialize)]
#[packet(0x03)]
pub struct CConnectedPong {
    ping_time: u64,
    pong_time: u64,
}

impl CConnectedPong {
    pub fn new(ping_time: u64, pong_time: u64) -> Self {
        Self {
            ping_time,
            pong_time,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[packet(0x10)]
pub struct CConnectionRequestAccepted {
    client_address: SocketAddress,
    system_index: u16,
    system_addresses: [SocketAddress; 10],
    requested_timestamp: u64,
    timestamp: u64,
}

impl CConnectionRequestAccepted {
    pub fn new(
        client_address: SocketAddress,
        system_index: u16,
        system_addresses: [SocketAddress; 10],
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
