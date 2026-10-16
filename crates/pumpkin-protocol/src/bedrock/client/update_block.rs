// Last verified for v2169

use pumpkin_macros::packet;
use pumpkin_util::math::position::BlockPos;

use crate::{codec::var_uint::VarUInt, serial::PacketWrite};

/// Updates a single block state client-side without retransmitting the entire chunk.
#[derive(PacketWrite)]
#[packet(21)]
pub struct CUpdateBlock {
    /// World coordinates where the block state is being updated.
    pub block_position: BlockPos,
    /// Runtime identifier corresponding to the new block state.
    pub block_runtime_id: VarUInt,
    /// Bitflags governing client-side update behaviors such as neighbor updates.
    pub flags: VarUInt,
    /// Target world layer for the block (e.g. 0 for primary blocks, 1 for waterlogging).
    pub layer: VarUInt,
}

impl CUpdateBlock {
    /// Constructs a block update packet targeting the default layer (0).
    #[must_use]
    pub const fn new(block_position: BlockPos, block_runtime_id: u32) -> Self {
        Self::with_layer(block_position, block_runtime_id, 0)
    }

    /// Constructs a block update packet targeting a specific layer with standard flags.
    #[must_use]
    pub const fn with_layer(block_position: BlockPos, block_runtime_id: u32, layer: u32) -> Self {
        Self {
            block_position,
            block_runtime_id: VarUInt(block_runtime_id),
            flags: VarUInt(0x3), // neighbors | network
            layer: VarUInt(layer),
        }
    }
}
