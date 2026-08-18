use pumpkin_macros::packet;
use pumpkin_util::math::position::BlockPos;

use crate::{codec::var_int::VarInt, serial::PacketWrite};

#[derive(Clone, Copy, PacketWrite)]
#[packet(43)]
pub struct CSetSpawnPosition {
    pub spawn_type: VarInt,
    pub position: BlockPos,
    pub dimension: VarInt,
    pub spawn_position: BlockPos,
}

impl CSetSpawnPosition {
    #[must_use]
    pub const fn new(
        spawn_type: i32,
        position: BlockPos,
        dimension: i32,
        spawn_position: BlockPos,
    ) -> Self {
        Self {
            spawn_type: VarInt(spawn_type),
            position,
            dimension: VarInt(dimension),
            spawn_position,
        }
    }
}
