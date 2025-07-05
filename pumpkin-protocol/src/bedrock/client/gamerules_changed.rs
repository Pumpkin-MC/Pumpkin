use pumpkin_macros::packet;
use serde::Serialize;

use crate::codec::var_uint::VarUInt;

#[derive(Serialize)]
#[packet(0x48)]
pub struct CGamerulesChanged {
    pub rule_data: GameRules,
}

impl CGamerulesChanged {
    pub fn new() -> Self {
        Self {
            rule_data: GameRules {
                list_size: VarUInt(0),
            },
        }
    }
}

#[derive(Serialize)]
pub struct GameRules {
    // TODO https://mojang.github.io/bedrock-protocol-docs/html/GameRulesChangedPacketData.html
    pub list_size: VarUInt,
}
