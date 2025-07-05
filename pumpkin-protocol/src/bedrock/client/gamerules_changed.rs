use pumpkin_macros::packet;
use serde::Serialize;

use crate::codec::var_uint::VarUInt;

#[derive(Serialize, Default)]
#[packet(0x48)]
pub struct CGamerulesChanged {
    pub rule_data: GameRules,
}

#[derive(Serialize, Default)]
pub struct GameRules {
    // TODO https://mojang.github.io/bedrock-protocol-docs/html/GameRulesChangedPacketData.html
    pub list_size: VarUInt,
}
