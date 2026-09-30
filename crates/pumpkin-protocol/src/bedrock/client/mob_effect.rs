// Last verified for v2169

use pumpkin_macros::packet;

use crate::{
    codec::{var_int::VarInt, var_ulong::VarULong},
    serial::PacketWrite,
};

#[derive(PacketWrite)]
#[packet(28)]
pub struct CMobEffect {
    pub target_runtime_id: VarULong,

    // TODO: Event enum
    pub event_id: u8,

    pub effect_id: VarInt,
    pub effect_amplifier: VarInt,
    pub show_particles: bool,
    pub effect_duration_ticks: VarInt,
    pub tick: VarULong,
    pub ambient: bool,
}

impl CMobEffect {
    pub const EVENT_ADD: u8 = 1;
    pub const EVENT_MODIFY: u8 = 2;
    pub const EVENT_REMOVE: u8 = 3;
}
