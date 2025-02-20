use pumpkin_data::packet::clientbound::PLAY_BLOCK_DESTRUCTION;
use pumpkin_util::math::position::BlockPos;

use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[client_packet(PLAY_BLOCK_DESTRUCTION)]
pub struct CSetBlockDestroyStage {
    entity_id: VarInt,
    location: BlockPos,
    destroy_stage: i8,
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
