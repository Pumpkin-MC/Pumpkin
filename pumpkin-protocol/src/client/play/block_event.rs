use pumpkin_data::packet::clientbound::PLAY_BLOCK_EVENT;
use pumpkin_util::math::position::BlockPos;

use pumpkin_macros::packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[packet(PLAY_BLOCK_EVENT)]
pub struct CBlockAction {
    location: BlockPos,
    action_id: u8,
    action_parameter: u8,
    block_type: VarInt,
}

impl CBlockAction {
    pub fn new(
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
