use std::net::SocketAddr;

use pumpkin_macros::packet;

use crate::{bedrock::RAKNET_MAGIC, serial::PacketWrite};

/// Sent by the server in response to `OpenConnectionRequest1` (`0x05`), negotiating security options, server GUID, and MTU.
///
/// Ref: <https://minecraft.wiki/w/RakNet#Open_Connection_Reply_1>
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

/// Sent by the server in response to `OpenConnectionRequest2` (`0x07`), confirming the connection setup and client address before establishing session state.
///
/// Ref: <https://minecraft.wiki/w/RakNet#Open_Connection_Reply_2>
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
    use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

    use super::{COpenConnectionReply1, COpenConnectionReply2};
    use crate::{bedrock::RAKNET_MAGIC, serial::PacketWrite};

    const SERVER_GUID: u64 = 0x0102_0304_0506_0708;
    const MTU: u16 = 1492;

    #[test]
    fn open_connection_reply_1_uses_raknet_byte_order() {
        let mut bytes = Vec::new();
        COpenConnectionReply1::new(SERVER_GUID, false, MTU)
            .write(&mut bytes)
            .unwrap();

        let mut expected = RAKNET_MAGIC.to_vec();
        expected.extend_from_slice(&SERVER_GUID.to_be_bytes());
        expected.push(0);
        expected.extend_from_slice(&MTU.to_be_bytes());
        assert_eq!(bytes, expected);
    }

    #[test]
    fn open_connection_reply_2_uses_raknet_address_and_byte_order() {
        let mut bytes = Vec::new();
        COpenConnectionReply2::new(
            SERVER_GUID,
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(192, 168, 0, 17), 19132)),
            MTU,
            false,
        )
        .write(&mut bytes)
        .unwrap();

        let mut expected = RAKNET_MAGIC.to_vec();
        expected.extend_from_slice(&SERVER_GUID.to_be_bytes());
        expected.extend_from_slice(&[4, !192, !168, !0, !17]);
        expected.extend_from_slice(&19132u16.to_be_bytes());
        expected.extend_from_slice(&MTU.to_be_bytes());
        expected.push(0);
        assert_eq!(bytes, expected);
    }
}
