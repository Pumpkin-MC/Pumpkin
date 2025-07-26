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
