use pumpkin_data::packet::clientbound::PLAY_BLOCK_EVENT;
use pumpkin_util::math::position::BlockPos;

use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::VarInt;

/// Triggers a physical block animation or sound effect.
///
/// This is used for simple block interactions that don't necessarily change
/// NBT data, such as chests opening/closing, pistons extending, or note blocks playing.
#[derive(Serialize)]
#[java_packet(PLAY_BLOCK_EVENT)]
pub struct CBlockEvent {
    /// The coordinates where the event occurs.
    pub location: BlockPos,
    /// The ID of the action to perform. Meaning varies by block type.
    pub action_id: u8,
    /// A parameter for the action (e.g., note pitch or instrument).
    pub action_parameter: u8,
    /// The block type ID (e.g., `minecraft:chest`).
    /// Note: This is the block ID, not the state ID.
    pub block_type: VarInt,
}

impl CBlockEvent {
    #[must_use]
    pub const fn new(
        location: BlockPos,
        action_id: u8,
        action_parameter: u8,
        block_type: VarInt,
    ) -> Self {
        Self {
            location,
            action_id,
            action_parameter,
            block_type,
        }
    }
}
