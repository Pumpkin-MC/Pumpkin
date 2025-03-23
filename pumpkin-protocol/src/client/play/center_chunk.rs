use pumpkin_data::packet::clientbound::PLAY_SET_CHUNK_CACHE_CENTER;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

use crate::VarInt;

#[derive(Serialize, Deserialize)]
#[packet(PLAY_SET_CHUNK_CACHE_CENTER)]
pub struct CCenterChunk {
    pub chunk_x: VarInt,
    pub chunk_z: VarInt,
}
