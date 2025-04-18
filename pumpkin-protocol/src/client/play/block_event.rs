use pumpkin_data::packet::clientbound::PLAY_BLOCK_EVENT;
use pumpkin_util::math::position::BlockPos;

use pumpkin_macros::packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[packet(PLAY_BLOCK_EVENT)]
pub struct CBlockEvent {
    location: BlockPos,
    r#type: u8,
    data: u8,
    block_type: VarInt,
}

impl CBlockEvent {
    pub fn new(location: BlockPos, r#type: u8, data: u8, block_type: VarInt) -> Self {
        Self {
            location,
            r#type,
            data,
            block_type,
        }
    }
}
