use pumpkin_macros::packet;

use crate::serial::{PacketRead, PacketWrite};

/// Sent by the client to request a specific chunk of a resource pack being downloaded.
#[derive(Clone, Debug, PartialEq, Eq, PacketRead, PacketWrite)]
#[packet(84)]
pub struct SResourcePackChunkRequest {
    /// Identifier of the resource pack being requested (`UUID` or `UUID_version`).
    pub pack_id: String,
    /// Sequential index of the requested chunk (starts at 0).
    pub chunk_index: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let packet = SResourcePackChunkRequest {
            pack_id: "test-uuid-123_1.0.0".to_string(),
            chunk_index: 3,
        };

        let mut bytes = Vec::new();
        packet.write(&mut bytes).unwrap();

        let decoded = SResourcePackChunkRequest::read(&mut bytes.as_slice()).unwrap();
        assert_eq!(decoded, packet);
    }
}
