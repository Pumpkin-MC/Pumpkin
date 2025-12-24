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
    pub motd_line_1: &'static str,
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
    pub fn new(time: u64, server_guid: u64, magic: [u8; 16], server_id: String) -> Self {
        Self {
            time,
            server_guid,
            magic,
            server_id,
        }
    }
}
