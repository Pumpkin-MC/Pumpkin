use pumpkin_macros::packet;

use crate::serial::PacketRead;

#[derive(PacketRead)]
#[packet(0x01)]
/// Used to request Server information like MOTD
pub struct SUnconnectedPing {
    #[serial(big_endian)]
    pub time: u64,
    pub magic: [u8; 16],
    #[serial(big_endian)]
    pub client_guid: u64,
}

#[derive(PacketRead)]
#[packet(0x02)]
/// Used to request Server information like MOTD when connection is open?
pub struct SUnconnectedPingOpenConnections {
    #[serial(big_endian)]
    pub time: u64,
    pub magic: [u8; 16],
    #[serial(big_endian)]
    pub client_guid: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bedrock::RAKNET_MAGIC;

    fn ping_bytes() -> Vec<u8> {
        // time (u64 BE), RakNet offline-message magic, client GUID (u64 BE)
        let mut bytes = 0x0000_0000_0001_e240u64.to_be_bytes().to_vec();
        bytes.extend_from_slice(&RAKNET_MAGIC);
        bytes.extend_from_slice(&0x0102_0304_0506_0708u64.to_be_bytes());
        bytes
    }

    #[test]
    fn unconnected_ping_reads_raknet_wire_format() {
        let bytes = ping_bytes();

        let packet = SUnconnectedPing::read(&mut bytes.as_slice()).unwrap();

        assert_eq!(packet.time, 123_456);
        assert_eq!(packet.magic, RAKNET_MAGIC);
        assert_eq!(packet.client_guid, 0x0102_0304_0506_0708);
    }

    #[test]
    fn unconnected_ping_open_connections_reads_raknet_wire_format() {
        let bytes = ping_bytes();

        let packet = SUnconnectedPingOpenConnections::read(&mut bytes.as_slice()).unwrap();

        assert_eq!(packet.time, 123_456);
        assert_eq!(packet.magic, RAKNET_MAGIC);
        assert_eq!(packet.client_guid, 0x0102_0304_0506_0708);
    }

    #[test]
    fn unconnected_ping_rejects_truncated_packet() {
        let bytes = &ping_bytes()[..20];

        assert!(SUnconnectedPing::read(&mut &bytes[..]).is_err());
    }
}
