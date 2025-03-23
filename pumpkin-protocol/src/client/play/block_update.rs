use pumpkin_data::packet::clientbound::PLAY_BLOCK_UPDATE;
use pumpkin_util::math::position::BlockPos;

use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

use crate::VarInt;

#[derive(Serialize, Deserialize)]
#[packet(PLAY_BLOCK_UPDATE)]
pub struct CBlockUpdate {
    location: BlockPos,
    block_id: VarInt,
}

impl CBlockUpdate {
    pub fn new(location: BlockPos, block_id: VarInt) -> Self {
        Self { location, block_id }
    }
}
