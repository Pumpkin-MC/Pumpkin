use crate::{codec::var_int::VarInt, serial::PacketWrite};
use pumpkin_macros::packet;

#[derive(PacketWrite)]
#[packet(42)]
pub struct CSetHealth {
    // https://mojang.github.io/bedrock-protocol-docs/html/SetHealthPacket.html
    pub health: VarInt,
}

impl CSetHealth {
    #[must_use]
    pub const fn new(health: i32) -> Self {
        Self {
            health: VarInt(health),
        }
    }
}
