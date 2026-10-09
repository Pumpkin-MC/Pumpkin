// Last verified for v2169

use pumpkin_macros::packet;

use crate::{codec::var_int::VarInt, serial::PacketWrite};

/// Sent by the server to synchronize the day-night cycle time (in ticks) with the client.
#[derive(PacketWrite)]
#[packet(10)]
pub struct CSetTime {
    /// In-game time of day in ticks.
    pub time: VarInt,
}

impl CSetTime {
    #[must_use]
    pub const fn new(time: i32) -> Self {
        Self { time: VarInt(time) }
    }
}
