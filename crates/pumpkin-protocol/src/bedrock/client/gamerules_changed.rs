use pumpkin_macros::packet;

use crate::{codec::var_uint::VarUInt, serial::PacketWrite};

/// Sent by the server to update gamerules on the client (e.g. `showcoordinates` or `dodaylightcycle`).
#[derive(PacketWrite)]
#[packet(72)]
pub struct CGamerulesChanged {
    /// List of game rules being updated.
    pub rule_data: Vec<GameRule>,
}

/// An individual gamerule entry with its identifier, edit permission, and typed value.
#[derive(PacketWrite)]
pub struct GameRule {
    /// Name of the gamerule in lowercase (e.g. `showcoordinates`).
    pub rule_name: String,
    /// Whether players with operator permissions can edit this gamerule in-game.
    pub rule_can_be_modified: bool,
    /// Value associated with the gamerule.
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
