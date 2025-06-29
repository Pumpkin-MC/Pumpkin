use pumpkin_data::packet::serverbound::PLAY_CHUNK_BATCH_RECEIVED;
use pumpkin_macros::packet;
use serde::Deserialize;

#[derive(Deserialize)]
#[packet(PLAY_CHUNK_BATCH_RECEIVED)]
pub struct SChunkBatch {
    pub chunks_per_tick: f32,
}
