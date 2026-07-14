use core::fmt;
use std::io::{Error, Write};

use pumpkin_macros::packet;

use crate::serial::PacketWrite;

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
        writer.write_all(&(self.server_id.len() as u16).to_be_bytes())?;
        writer.write_all(self.server_id.as_bytes())
    }
}

pub struct ServerInfo {
    /// (BE or MCEE for Education Edition)
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

impl CUnconnectedPong {
    #[must_use]
    pub const fn new(time: u64, server_guid: u64, magic: [u8; 16], server_id: String) -> Self {
        Self {
            time,
            server_guid,
            magic,
            server_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bedrock::RAKNET_MAGIC;

    #[test]
    fn unconnected_pong_writes_raknet_wire_format() {
        let mut bytes = Vec::new();
        CUnconnectedPong::new(
            123_456,
            0x0102_0304_0506_0708,
            RAKNET_MAGIC,
            "MCPE;Pumpkin".to_string(),
        )
        .write(&mut bytes)
        .unwrap();

        // time (u64 BE), server GUID (u64 BE), magic, u16 BE length-prefixed server ID
        let mut expected = 123_456u64.to_be_bytes().to_vec();
        expected.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        expected.extend_from_slice(&RAKNET_MAGIC);
        expected.extend_from_slice(&12u16.to_be_bytes());
        expected.extend_from_slice(b"MCPE;Pumpkin");
        assert_eq!(bytes, expected);
    }

    #[test]
    fn server_info_formats_bedrock_motd_string() {
        let info = ServerInfo {
            edition: "MCPE",
            motd_line_1: "A Blazingly fast Server".to_string(),
            protocol_version: 819,
            version_name: "1.21.90",
            player_count: 3,
            max_player_count: 20,
            server_unique_id: 0x0102_0304_0506_0708,
            motd_line_2: "world".to_string(),
            game_mode: "Survival",
            game_mode_numeric: 1,
            port_ipv4: 19132,
            port_ipv6: 19133,
        };

        assert_eq!(
            info.to_string(),
            "MCPE;A Blazingly fast Server;819;1.21.90;3;20;72623859790382856;world;Survival;1;19132;19133;0;"
        );
    }
}
