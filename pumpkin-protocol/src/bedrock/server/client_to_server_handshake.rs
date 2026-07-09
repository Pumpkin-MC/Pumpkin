use pumpkin_macros::packet;

use crate::serial::PacketRead;

#[derive(PacketRead)]
#[packet(0x04)]
pub struct SClientToServerHandshake;
