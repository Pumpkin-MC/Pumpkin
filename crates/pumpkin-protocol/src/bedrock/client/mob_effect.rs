use pumpkin_macros::packet;

use crate::{
    codec::{var_int::VarInt, var_ulong::VarULong},
    serial::PacketWrite,
};

#[derive(PacketWrite)]
#[packet(28)]
pub struct CMobEffect {
    pub runtime_entity_id: VarULong,
    pub event_id: u8,
    pub effect_id: VarInt,
    pub amplifier: VarInt,
    pub particles: bool,
    pub duration: VarInt,
    pub tick: VarULong,
    pub ambient: bool,
}

impl CMobEffect {
    pub const EVENT_ADD: u8 = 1;
    pub const EVENT_MODIFY: u8 = 2;
    pub const EVENT_REMOVE: u8 = 3;

    #[expect(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        runtime_entity_id: VarULong,
        event_id: u8,
        effect_id: VarInt,
        amplifier: VarInt,
        particles: bool,
        duration: VarInt,
        tick: VarULong,
        ambient: bool,
    ) -> Self {
        Self {
            runtime_entity_id,
            event_id,
            effect_id,
            amplifier,
            particles,
            duration,
            tick,
            ambient,
        }
    }
}
