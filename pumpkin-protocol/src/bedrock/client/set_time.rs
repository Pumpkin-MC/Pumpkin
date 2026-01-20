use pumpkin_macros::packet;

use crate::{codec::var_int::VarInt, serial::PacketWrite};

#[derive(PacketWrite)]
#[packet(10)]
pub struct CSetTime {
    pub time: VarInt,
}

impl CSetTime {
    #[must_use] 
    pub const fn new(time: i32) -> Self {
        Self { time: VarInt(time) }
    }
}
