use pumpkin_data::packet::clientbound::PLAY_BLOCK_DESTRUCTION;
use pumpkin_util::math::position::BlockPos;

use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::VarInt;

/// Updates the visual "breaking" progress of a block for all clients.
///
/// This packet controls the overlay of cracks that appear on a block when
/// it is being mined. It is often used to show other players' mining progress.
#[derive(Serialize)]
#[java_packet(PLAY_BLOCK_DESTRUCTION)]
pub struct CSetBlockDestroyStage {
    /// A unique ID for this destruction instance. Usually the miner's Entity ID.
    /// If multiple entities mine the same block, they must use different IDs.
    pub entity_id: VarInt,
    /// The coordinates of the block being destroyed.
    pub location: BlockPos,
    /// The destruction stage, typically a value from 0 to 9.
    /// Any value outside 0-9 (like -1) will remove the destruction overlay.
    pub destroy_stage: i8,
}

impl CSetBlockDestroyStage {
    pub fn new(entity_id: VarInt, location: BlockPos, destroy_stage: i8) -> Self {
        Self {
            entity_id,
            location,
            destroy_stage,
        }
    }
}
