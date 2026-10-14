// Last verified for v2169

use pumpkin_macros::packet;

use crate::{codec::var_uint::VarUInt, serial::PacketWrite};

/// Synchronizes world difficulty level to the client.
#[derive(PacketWrite)]
#[packet(60)]
pub struct CSetDifficulty {
    /// World difficulty value (0: Peaceful, 1: Easy, 2: Normal, 3: Hard).
    pub difficulty: VarUInt,
}
