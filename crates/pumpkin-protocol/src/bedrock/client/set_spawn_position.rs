// Last verified for v2169

use pumpkin_macros::packet;
use pumpkin_util::math::position::BlockPos;

use crate::{codec::var_int::VarInt, serial::PacketWrite};

/// Updates the player or world spawn coordinates, affecting respawning and compass orientation.
#[derive(Clone, Copy, PacketWrite)]
#[packet(43)]
pub struct CSetSpawnPosition {
    /// Determines whether this update configures a player-specific or world spawn location.
    pub spawn_position_type: SpawnPositionType,
    /// Spawn coordinates being set.
    pub block_position: BlockPos,
    /// Dimension identifier associated with this spawn position.
    pub dimension_type: VarInt,
    /// World spawn coordinates.
    pub spawn_block_pos: BlockPos,
}

/// Scope of the spawn position update.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PacketWrite)]
#[repr(i32)]
#[serial(varint)]
pub enum SpawnPositionType {
    /// Sets an individual player respawn point (e.g. bed or respawn anchor).
    PlayerRespawn,
    /// Sets the global world spawn point, also updating compass direction.
    WorldRespawn,
}
