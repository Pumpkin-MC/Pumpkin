use pumpkin_data::packet::clientbound::PLAY_CHUNK_BATCH_START;
use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(PLAY_CHUNK_BATCH_START)]
pub struct CChunkBatchStart;
