use pumpkin_macros::packet;

use crate::{codec::var_uint::VarUInt, serial::PacketWrite};

#[derive(PacketWrite)]
#[packet(72)]
pub struct CGamerulesChanged {
    pub rule_data: Vec<GameRule>,
}

#[derive(PacketWrite)]
pub struct GameRule {
    pub rule_name: String,
    pub rule_can_be_modified: bool,
    pub rule_value: RuleValue,
}

// TODO: flesh out RuleValue
pub enum RuleValue {
    Null,
}

impl PacketWrite for RuleValue {
    fn write<W: std::io::prelude::Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        VarUInt(0).write(writer)
    }
}
