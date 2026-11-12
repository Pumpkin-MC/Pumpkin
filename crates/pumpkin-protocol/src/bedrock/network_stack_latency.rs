// Last verified for v2169
// Bidirectional. The server sends `from_server = true`, then the client echoes it. A client
// may also send `from_server = false` as its own ping, which the server echoes.

use pumpkin_macros::packet;

use crate::serial::{PacketRead, PacketWrite};

#[derive(Debug, Clone, Copy, PacketRead, PacketWrite)]
#[packet(115)]
pub struct NetworkStackLatency {
    pub timestamp: u64,
    pub from_server: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packet::Packet;
    use crate::serial::{PacketRead, PacketWrite};

    #[test]
    fn packet_id_is_ping() {
        assert_eq!(NetworkStackLatency::PACKET_ID, 115);
    }

    #[test]
    fn round_trips() {
        let packet = NetworkStackLatency {
            timestamp: 1_700_000_000_000_000,
            from_server: true,
        };
        let mut buf = Vec::new();
        packet.write(&mut buf).unwrap();
        let decoded = NetworkStackLatency::read(&mut buf.as_slice()).unwrap();
        assert_eq!(decoded.timestamp, packet.timestamp);
        assert!(decoded.from_server);
    }
}
