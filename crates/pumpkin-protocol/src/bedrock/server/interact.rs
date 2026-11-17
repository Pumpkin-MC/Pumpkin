// Last verified for v2169

use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

use crate::{codec::var_ulong::VarULong, serial::PacketRead};

/// Sent by the client when interacting with an entity (mounting, NPC dialogue, opening inventory).
#[derive(Debug, PacketRead)]
#[packet(33)]
pub struct SInteract {
    /// Type of interaction performed.
    pub action: Action,
    /// Runtime entity ID of the target entity.
    pub target_runtime_id: VarULong,
    /// Interaction hit position vector, present for mouse/touch interactions.
    pub position: Option<Vector3<f32>>,
}

/// Action type performed in an entity interaction.
#[derive(Debug, PacketRead)]
#[repr(u8)]
pub enum Action {
    Invalid = 0,
    /// Stop riding a vehicle entity.
    StopRiding = 3,
    /// Continuous interaction position update.
    InteractUpdate = 4,
    /// Open NPC dialog interface.
    NpcOpen = 5,
    /// Open entity inventory (e.g. horse or llama).
    OpenInventory = 6,
}
