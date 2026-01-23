use pumpkin_data::packet::clientbound::PLAY_BLOCK_ENTITY_DATA;
use pumpkin_macros::packet;
use pumpkin_util::math::position::BlockPos;
use serde::Serialize;

use crate::{VarInt, ser::network_serialize_no_prefix};

#[derive(Serialize)]
#[packet(PLAY_BLOCK_ENTITY_DATA)]
pub struct CBlockEntityData {
    pub location: BlockPos,
    pub r#type: VarInt,
    #[serde(serialize_with = "network_serialize_no_prefix")]
    pub nbt_data: Box<[u8]>,
}

impl CBlockEntityData {
    #[must_use]
    pub const fn new(location: BlockPos, r#type: VarInt, nbt_data: Box<[u8]>) -> Self {
        Self {
            location,
            r#type,
            nbt_data,
        }
    }
}
