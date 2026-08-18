use pumpkin_macros::packet;

use crate::{codec::var_uint::VarUInt, serial::PacketWrite};

#[derive(PacketWrite)]
#[packet(60)]
pub struct CSetDifficulty {
    pub difficulty: VarUInt,
}

impl CSetDifficulty {
    #[must_use]
    pub const fn new(difficulty: u32) -> Self {
        Self {
            difficulty: VarUInt(difficulty),
        }
    }
}
