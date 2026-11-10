// Last verified for v2169

use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

use crate::{codec::var_ulong::VarULong, serial::PacketWrite};

/// Sent by the server to set the movement velocity of an entity (e.g. knockback or explosion velocity).
#[derive(PacketWrite)]
#[packet(40)]
pub struct CSetActorMotion {
    /// Runtime entity ID of the target actor.
    pub target_runtime_id: VarULong,
    /// 3D velocity vector applied to the entity.
    pub motion: Vector3<f32>,
    /// Simulation tick number for server authoritative syncing.
    pub tick: VarULong,
}
