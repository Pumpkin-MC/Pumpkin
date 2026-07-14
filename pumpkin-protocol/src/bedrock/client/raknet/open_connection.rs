use std::net::SocketAddr;

use pumpkin_macros::packet;

use crate::{bedrock::RAKNET_MAGIC, serial::PacketWrite};

#[derive(PacketWrite)]
#[packet(0x06)]
pub struct COpenConnectionReply1 {
    magic: [u8; 16],
    #[serial(big_endian)]
    server_guid: u64,
    has_server_security: bool,
    // Only write when has_server_security
    // cookie: u32,
    #[serial(big_endian)]
    mtu: u16,
}

impl COpenConnectionReply1 {
    #[must_use]
    pub const fn new(server_guid: u64, has_server_security: bool, mtu: u16) -> Self {
        Self {
            magic: RAKNET_MAGIC,
            server_guid,
            has_server_security,
            // cookie,
            mtu,
        }
    }
}

#[derive(PacketWrite)]
#[packet(0x08)]
pub struct COpenConnectionReply2 {
    magic: [u8; 16],
    #[serial(big_endian)]
    server_guid: u64,
    client_address: SocketAddr,
    #[serial(big_endian)]
    mtu: u16,
    security: bool,
}

impl COpenConnectionReply2 {
    #[must_use]
    pub const fn new(
        server_guid: u64,
        client_address: SocketAddr,
        mtu: u16,
        security: bool,
    ) -> Self {
        Self {
            magic: RAKNET_MAGIC,
            server_guid,
            client_address,
            mtu,
            security,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_connection_reply_1_uses_raknet_wire_format() {
        let mut bytes = Vec::new();
        COpenConnectionReply1::new(0x0102_0304_0506_0708, false, 1400)
            .write(&mut bytes)
            .unwrap();

        let mut expected = RAKNET_MAGIC.to_vec();
        expected.extend_from_slice(&[
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x00, 0x05, 0x78,
        ]);
        assert_eq!(bytes, expected);
    }

    #[test]
    fn open_connection_reply_2_uses_raknet_wire_format() {
        let mut bytes = Vec::new();
        COpenConnectionReply2::new(
            0x0102_0304_0506_0708,
            SocketAddr::from(([192, 0, 2, 1], 19132)),
            1400,
            false,
        )
        .write(&mut bytes)
        .unwrap();

        let mut expected = RAKNET_MAGIC.to_vec();
        expected.extend_from_slice(&[
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x04, 0x3f, 0xff, 0xfd, 0xfe, 0x4a,
            0xbc, 0x05, 0x78, 0x00,
        ]);
        assert_eq!(bytes, expected);
    }
}
