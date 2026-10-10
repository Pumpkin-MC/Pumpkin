// Last verified for v2169

use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

use crate::{codec::var_int::VarInt, serial::PacketWrite};

/// Broadcasts visual or auditory level-wide events, such as particles or block breaking.
#[derive(PacketWrite)]
#[packet(25)]
pub struct CLevelEvent {
    /// Identifier corresponding to the specific level event action.
    pub event_id: VarInt,
    /// World coordinates where the event originates or particle effect is centered.
    pub position: Vector3<f32>,
    /// Type-specific auxiliary parameter associated with the event (e.g. block runtime ID).
    pub data: VarInt,
}

/// Identifiers for common level-wide events and particle effects.
#[repr(i32)]
pub enum LevelEvent {
    // There are hundreds of these, adding only what we need for now
    /// Emits block break particles at the target position.
    ParticlesDestroyBlock = 2001,
    /// Initiates a block cracking animation.
    BlockStartBreak = 3600,
    /// Cancels or completes a block cracking animation.
    BlockStopBreak = 3601,
    /// Advances the crack stage of a block cracking animation.
    BlockUpdateBreak = 3602,
}
