use crate::serial::PacketWrite;
use pumpkin_macros::packet;
use pumpkin_util::GameMode;

#[derive(PacketWrite)]
#[packet(62)]
pub struct CSetPlayerGamemode {
    pub gamemode: GameMode,
}
