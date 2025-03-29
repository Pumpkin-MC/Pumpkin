use pumpkin_data::packet::clientbound::PLAY_CHUNK_BATCH_START;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[packet(PLAY_CHUNK_BATCH_START)]
pub struct CChunkBatchStart;
