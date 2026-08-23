use core::fmt;
use std::io::{Error, Write};

use pumpkin_macros::packet;

use crate::serial::{PacketRead, PacketWrite};

pub const OFFLINE_MESSAGE_MAGIC: [u8; 16] = [
    0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78,
];

#[derive(PacketRead)]
#[packet(0x01)]
pub struct SUnconnectedPing {
    #[serial(big_endian)]
    pub time: u64,
    pub magic: [u8; 16],
    #[serial(big_endian)]
    pub client_guid: u64,
}

#[derive(PacketRead)]
#[packet(0x02)]
pub struct SUnconnectedPingOpenConnections {
    #[serial(big_endian)]
    pub time: u64,
    pub magic: [u8; 16],
    #[serial(big_endian)]
    pub client_guid: u64,
}

#[packet(0x1c)]
pub struct CUnconnectedPong {
    time: u64,
    server_guid: u64,
    magic: [u8; 16],
    server_id: String,
}

impl CUnconnectedPong {
    #[must_use]
    pub const fn new(time: u64, server_guid: u64, server_id: String) -> Self {
        Self {
            time,
            server_guid,
            magic: OFFLINE_MESSAGE_MAGIC,
            server_id,
        }
    }
}

impl PacketWrite for CUnconnectedPong {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.time.write_be(writer)?;
        self.server_guid.write_be(writer)?;
        writer.write_all(&self.magic)?;
        let length = u16::try_from(self.server_id.len())
            .map_err(|_| Error::other("Bedrock server advertisement is too long"))?;
        writer.write_all(&length.to_be_bytes())?;
        writer.write_all(self.server_id.as_bytes())
    }
}

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
            magic: OFFLINE_MESSAGE_MAGIC,
            server_guid,
        }
    }
}

pub struct ServerInfo<'a> {
    pub motd: &'a str,
    pub protocol: u32,
    pub version: &'static str,
    pub players: i32,
    pub max_players: u32,
    pub server_guid: u64,
    pub level_name: &'a str,
    pub game_mode: &'static str,
    pub game_mode_id: u32,
    pub ipv4_port: u16,
    pub ipv6_port: u16,
}

impl fmt::Display for ServerInfo<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MCPE;{};{};{};{};{};{};{};{};{};{};{};0;",
            self.motd,
            self.protocol,
            self.version,
            self.players,
            self.max_players,
            self.server_guid,
            self.level_name,
            self.game_mode,
            self.game_mode_id,
            self.ipv4_port,
            self.ipv6_port
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_vanilla_26_40_advertisement() {
        let info = ServerInfo {
            motd: "Pumpkin",
            protocol: 2168,
            version: "1.26.40",
            players: 2,
            max_players: 20,
            server_guid: 42,
            level_name: "world",
            game_mode: "Creative",
            game_mode_id: 1,
            ipv4_port: 19132,
            ipv6_port: 19133,
        };

        assert_eq!(
            info.to_string(),
            "MCPE;Pumpkin;2168;1.26.40;2;20;42;world;Creative;1;19132;19133;0;"
        );
    }

    #[test]
    fn encodes_raknet_incompatible_protocol_response() {
        let mut response = Vec::new();
        CIncompatibleProtocolVersion::new(12, 42)
            .write(&mut response)
            .unwrap();

        assert_eq!(response[0], 12);
        assert_eq!(response[1..17], OFFLINE_MESSAGE_MAGIC);
        assert_eq!(response[17..], 42u64.to_be_bytes());
    }
}
