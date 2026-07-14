use std::net::SocketAddr;

use pumpkin_macros::packet;

use crate::serial::PacketRead;

#[derive(PacketRead)]
#[packet(0x05)]
/// The client sends this when attempting to join the server
pub struct SOpenConnectionRequest1 {
    pub magic: [u8; 16],
    pub protocol_version: u8,
    #[serial(big_endian)]
    pub mtu: u16,
}

#[derive(PacketRead)]
#[packet(0x07)]
pub struct SOpenConnectionRequest2 {
    pub magic: [u8; 16],
    pub server_address: SocketAddr,
    #[serial(big_endian)]
    pub mtu: u16,
    #[serial(big_endian)]
    pub client_guid: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{bedrock::RAKNET_MAGIC, serial::PacketRead};

    #[test]
    fn open_connection_request_2_reads_raknet_wire_format() {
        let mut bytes = RAKNET_MAGIC.to_vec();
        bytes.extend_from_slice(&[
            0x04, 0x3f, 0xff, 0xfd, 0xfe, 0x4a, 0xbc, 0x05, 0x78, 0x01, 0x02, 0x03, 0x04, 0x05,
            0x06, 0x07, 0x08,
        ]);

        let packet = SOpenConnectionRequest2::read(&mut bytes.as_slice()).unwrap();

        assert_eq!(packet.magic, RAKNET_MAGIC);
        assert_eq!(
            packet.server_address,
            SocketAddr::from(([192, 0, 2, 1], 19132))
        );
        assert_eq!(packet.mtu, 1400);
        assert_eq!(packet.client_guid, 0x0102_0304_0506_0708);
    }
}
