use core::fmt;
use std::io::{Error, Write};

use pumpkin_macros::packet;

use crate::serial::{PacketRead, PacketWrite};

pub const OFFLINE_MESSAGE_MAGIC: [u8; 16] = [
    0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78,
];

/// Requests Bedrock server-list status without opening a game connection.
#[derive(PacketRead)]
#[packet(0x01)]
pub struct SUnconnectedPing {
    #[serial(big_endian)]
    pub time: u64,
    pub magic: [u8; 16],
    #[serial(big_endian)]
    pub client_guid: u64,
}

/// Requests status only when the server is accepting connections.
#[derive(PacketRead)]
#[packet(0x02)]
pub struct SUnconnectedPingOpenConnections {
    #[serial(big_endian)]
    pub time: u64,
    pub magic: [u8; 16],
    #[serial(big_endian)]
    pub client_guid: u64,
}

/// Returns the server's MOTD and status to the Bedrock server list.
#[packet(0x1c)]
pub struct CUnconnectedPong {
    time: u64,
    server_guid: u64,
    magic: [u8; 16],
    server_id: String,
}

impl PacketWrite for CUnconnectedPong {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.time.write_be(writer)?;
        self.server_guid.write_be(writer)?;
        writer.write_all(&self.magic)?;
        let length = u16::try_from(self.server_id.len())
            .map_err(|_| Error::other("Bedrock server status is too long"))?;
        writer.write_all(&length.to_be_bytes())?;
        writer.write_all(self.server_id.as_bytes())
    }
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

pub struct ServerInfo {
    pub edition: &'static str,
    pub motd_line_1: String,
    pub protocol_version: u32,
    pub version_name: &'static str,
    pub player_count: i32,
    pub max_player_count: u32,
    pub server_unique_id: u64,
    pub motd_line_2: String,
    pub game_mode: &'static str,
    pub game_mode_numeric: u32,
    pub port_ipv4: u16,
    pub port_ipv6: u16,
}

impl fmt::Display for ServerInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{};{};{};{};{};{};{};{};{};{};{};{};0;",
            self.edition,
            self.motd_line_1,
            self.protocol_version,
            self.version_name,
            self.player_count,
            self.max_player_count,
            self.server_unique_id,
            self.motd_line_2,
            self.game_mode,
            self.game_mode_numeric,
            self.port_ipv4,
            self.port_ipv6
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_bedrock_server_list_status() {
        let status = ServerInfo {
            edition: "MCPE",
            motd_line_1: "Pumpkin".into(),
            protocol_version: 1000,
            version_name: "1.26.40",
            player_count: 2,
            max_player_count: 20,
            server_unique_id: 42,
            motd_line_2: "world".into(),
            game_mode: "Survival",
            game_mode_numeric: 0,
            port_ipv4: 19132,
            port_ipv6: 19133,
        };

        assert_eq!(
            status.to_string(),
            "MCPE;Pumpkin;1000;1.26.40;2;20;42;world;Survival;0;19132;19133;0;"
        );
    }
}
