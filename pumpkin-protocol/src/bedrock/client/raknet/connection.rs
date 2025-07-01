use crate::ser::network_serialize_no_prefix;
use pumpkin_macros::packet;
use serde::Serialize;

use crate::codec::socket_address::SocketAddress;

#[derive(Serialize)]
#[packet(0x10)]
pub struct CConnectionRequestAccepted {
    client_address: SocketAddress,
    system_index: u16,
    #[serde(serialize_with = "network_serialize_no_prefix")]
    system_addresses: Vec<SocketAddress>,
    requested_timestamp: u64,
    timestamp: u64,
}

impl CConnectionRequestAccepted {
    pub fn new(
        client_address: SocketAddress,
        system_index: u16,
        system_addresses: Vec<SocketAddress>,
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
