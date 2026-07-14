use crate::{bedrock::RAKNET_MAGIC, serial::PacketWrite};
use pumpkin_macros::packet;

#[derive(PacketWrite)]
#[packet(0x19)]
pub struct CIncompatibleProtocolVersion {
    protocol_version: u8,
    magic: [u8; 16],
    #[serial(big_endian)]
    server_guid: u64,
}

impl CIncompatibleProtocolVersion {
    #[must_use]
    pub const fn new(protocol_version: u8, server_guid: u64) -> Self {
        Self {
            protocol_version,
            magic: RAKNET_MAGIC,
            server_guid,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incompatible_protocol_version_uses_network_byte_order() {
        let mut bytes = Vec::new();
        CIncompatibleProtocolVersion::new(11, 0x0102_0304_0506_0708)
            .write(&mut bytes)
            .unwrap();

        let mut expected = vec![11];
        expected.extend_from_slice(&RAKNET_MAGIC);
        expected.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(bytes, expected);
    }
}
