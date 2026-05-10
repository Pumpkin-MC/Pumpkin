use pumpkin_macros::packet;

use crate::serial::PacketRead;

#[derive(PacketRead)]
#[packet(0xC1)]
pub struct SRequestNetworkSettings {
    #[serial(big_endian)]
    pub protocol_version: i32,
}
