// Last verified for v2169

use pumpkin_macros::packet;
use pumpkin_util::math::position::BlockPos;

use crate::{codec::var_long::VarLong, serial::PacketWrite};

/// Sent by the server to open an inventory container screen (chest, furnace, anvil) on the client.
#[derive(PacketWrite)]
#[packet(46)]
pub struct CContainerOpen {
    /// Window ID assigned to the opened container session.
    pub container_id: u8,
    /// Container inventory type (chest, hopper, dispenser, etc.).
    pub container_type: u8,
    /// Block coordinates of the container block entity in world space.
    pub position: BlockPos,
    /// Entity ID if the container belongs to an entity (e.g. horse or donkey chest).
    pub target_entity_id: VarLong,
}
