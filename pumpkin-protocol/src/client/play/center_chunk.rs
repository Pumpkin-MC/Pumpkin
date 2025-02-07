use pumpkin_data::packet::clientbound::PLAY_SET_CHUNK_CACHE_CENTER;
use pumpkin_macros::client_packet;

use crate::VarInt;

#[derive(serde::Serialize)]
#[client_packet(PLAY_SET_CHUNK_CACHE_CENTER)]
pub struct CCenterChunk {
    pub chunk_x: VarInt,
    pub chunk_z: VarInt,
}
