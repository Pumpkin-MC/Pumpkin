use pumpkin_macros::packet;

use crate::{codec::var_uint::VarUInt, serial::PacketWrite};

#[derive(PacketWrite, Default)]
#[packet(0x48)]
pub struct CGamerulesChanged {
    pub rule_data: GameRules,
}

#[derive(PacketWrite, Default)]
pub struct GameRules {
    // TODO https://mojang.github.io/bedrock-protocol-docs/html/GameRulesChangedPacketData.html
    pub list_size: VarUInt,
}
