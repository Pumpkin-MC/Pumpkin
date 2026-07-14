use std::net::SocketAddr;

use pumpkin_macros::packet;

use crate::{bedrock::RAKNET_MAGIC, serial::PacketWrite};
#[derive(PacketWrite)]
#[packet(0x03)]
pub struct CConnectedPong {
    #[serial(big_endian)]
    ping: u64,
    #[serial(big_endian)]
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
    #[serial(big_endian)]
    system_index: u16,
    system_addresses: [SocketAddr; 10],
    #[serial(big_endian)]
    requested_timestamp: u64,
    #[serial(big_endian)]
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
    #[serial(big_endian)]
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
    #[serial(big_endian)]
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
    #[serial(big_endian)]
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
    #[serial(big_endian)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connected_pong_uses_network_byte_order() {
        let mut bytes = Vec::new();
        CConnectedPong::new(0x0102_0304_0506_0708, 0x1112_1314_1516_1718)
            .write(&mut bytes)
            .unwrap();

        assert_eq!(
            bytes,
            [
                0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16,
                0x17, 0x18,
            ]
        );
    }

    #[test]
    fn connection_request_accepted_uses_raknet_wire_format() {
        let client_address = SocketAddr::from(([127, 0, 0, 1], 19132));
        let system_address = SocketAddr::from(([0, 0, 0, 0], 0));
        let mut bytes = Vec::new();
        CConnectionRequestAccepted::new(
            client_address,
            0x0102,
            [system_address; 10],
            0x0304_0506_0708_090a,
            0x1112_1314_1516_1718,
        )
        .write(&mut bytes)
        .unwrap();

        let mut expected = vec![4, 0x80, 0xff, 0xff, 0xfe, 0x4a, 0xbc, 0x01, 0x02];
        for _ in 0..10 {
            expected.extend_from_slice(&[4, 0xff, 0xff, 0xff, 0xff, 0, 0]);
        }
        expected.extend_from_slice(&[
            0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16,
            0x17, 0x18,
        ]);

        assert_eq!(bytes, expected);
    }
}
