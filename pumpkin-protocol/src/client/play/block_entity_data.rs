use pumpkin_data::packet::clientbound::PLAY_BLOCK_ENTITY_DATA;
use pumpkin_macros::client_packet;
use pumpkin_util::math::position::BlockPos;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[client_packet(PLAY_BLOCK_ENTITY_DATA)]
pub struct CBlockEntityData {
    location: BlockPos,
    r#type: VarInt,
    nbt_data: Box<[u8]>,
}

impl CBlockEntityData {
    pub fn new(location: BlockPos, r#type: VarInt, nbt_data: Box<[u8]>) -> Self {
        Self {
            location,
            r#type,
            nbt_data,
        }
    }
}
