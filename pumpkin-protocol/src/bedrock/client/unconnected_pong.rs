use core::fmt;

use pumpkin_macros::packet;
use serde::Serialize;

use crate::codec::ascii_string::AsciiString;

#[derive(Serialize)]
#[packet(0x1c)]
pub struct CUnconnectedPong {
    time: i64,
    server_guid: u64,
    magic: [u8; 16],
    server_id: AsciiString,
}

pub struct ServerInfo {
    /// (MCPE or MCEE for Education Edition)
    pub edition: &'static str,
    pub motd_line_1: &'static str,
    pub protocol_version: u32,
    pub version_name: &'static str,
    pub player_count: i32,
    pub max_player_count: u32,
    pub server_unique_id: u64,
    pub motd_line_2: &'static str,
    pub game_mode: &'static str,
    pub game_mode_numeric: u32,
    pub port_ipv4: u16,
    pub port_ipv6: u16,
}

impl fmt::Display for ServerInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{};{};{};{};{};{};{};{};{};{};{};{}",
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
    pub fn new(time: i64, server_guid: u64, magic: [u8; 16], server_id: AsciiString) -> Self {
        Self {
            time,
            server_guid,
            magic,
            server_id,
        }
    }
}
