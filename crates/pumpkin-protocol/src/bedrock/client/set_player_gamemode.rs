// Last verified for v2169

use crate::{bedrock::client::GameType, serial::PacketWrite};
use pumpkin_macros::packet;

#[derive(PacketWrite)]
#[packet(62)]
pub struct CSetPlayerGameType {
    pub player_game_type: GameType,
}
