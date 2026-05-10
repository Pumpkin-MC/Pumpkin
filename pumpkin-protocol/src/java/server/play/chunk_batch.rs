use pumpkin_data::packet::serverbound::PLAY_CHUNK_BATCH_RECEIVED;
use pumpkin_macros::java_packet;
use serde::Deserialize;

#[derive(Deserialize)]
#[java_packet(PLAY_CHUNK_BATCH_RECEIVED)]
pub struct SChunkBatch {
    pub chunks_per_tick: f32,
}
