use pumpkin_data::packet::clientbound::PLAY_BLOCK_UPDATE;
use pumpkin_util::math::position::BlockPos;

use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::VarInt;

/// Updates a single block state at a specific location in the world.
///
/// This is the most common way to sync world changes to the client, such as
/// when a player places a block, a fluid flows, or a redstone component toggles.
#[derive(Serialize)]
#[java_packet(PLAY_BLOCK_UPDATE)]
pub struct CBlockUpdate {
    /// The world coordinates of the block being updated.
    pub location: BlockPos,
    /// The new block state ID.
    pub state_id: VarInt,
}

impl CBlockUpdate {
    #[must_use]
    pub fn new(location: BlockPos, state_id: VarInt) -> Self {
        Self { location, state_id }
    }
}
