use pumpkin_data::packet::clientbound::PLAY_BLOCK_UPDATE;
use pumpkin_util::math::position::BlockPos;

use pumpkin_macros::packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[packet(PLAY_BLOCK_UPDATE)]
pub struct CBlockUpdate<'a> {
    location: &'a BlockPos,
    block_id: VarInt,
}

impl<'a> CBlockUpdate<'a> {
    pub fn new(location: &'a BlockPos, block_id: VarInt) -> Self {
        Self { location, block_id }
    }
}
