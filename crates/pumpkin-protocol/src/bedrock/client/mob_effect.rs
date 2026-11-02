// Last verified for v2169

use pumpkin_macros::packet;

use crate::{
    codec::{var_int::VarInt, var_ulong::VarULong},
    serial::PacketWrite,
};

/// Sent by the server to add, modify, or remove potion/status effects on an entity.
#[derive(PacketWrite)]
#[packet(28)]
pub struct CMobEffect {
    /// Runtime entity ID of the target actor.
    pub target_runtime_id: VarULong,

    // TODO: Event enum
    /// Effect action event (Add, Modify, or Remove).
    pub event_id: u8,

    /// Status effect type identifier.
    pub effect_id: VarInt,
    /// Amplifier/potency level of the effect (e.g. 0 for Speed I, 1 for Speed II).
    pub effect_amplifier: VarInt,
    /// Whether swirl particles should be displayed around the entity.
    pub show_particles: bool,
    /// Duration of the effect in ticks.
    pub effect_duration_ticks: VarInt,
    /// Simulation tick number for server authoritative syncing.
    pub tick: VarULong,
    /// Whether the effect is ambient (e.g. from a beacon).
    pub ambient: bool,
}

impl CMobEffect {
    /// Action adding a new status effect to the entity.
    pub const EVENT_ADD: u8 = 1;
    /// Action modifying duration or amplifier of an existing effect.
    pub const EVENT_MODIFY: u8 = 2;
    /// Action removing an active status effect from the entity.
    pub const EVENT_REMOVE: u8 = 3;
}
